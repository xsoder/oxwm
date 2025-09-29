use crate::keyboard::{self, Arg, KeyAction};
use crate::layout::Layout;
use crate::layout::tiling::TilingLayout;

use anyhow::Result;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

pub struct WindowManager {
    connection: RustConnection,
    screen_number: usize,
    root: Window,
    screen: Screen,
    windows: Vec<Window>,
    focused_window: Option<Window>,
    layout: Box<dyn Layout>,
}

impl WindowManager {
    pub fn new() -> Result<Self> {
        let (connection, screen_number) = x11rb::connect(None)?;
        let root = connection.setup().roots[screen_number].root;
        let screen = connection.setup().roots[screen_number].clone();

        connection
            .change_window_attributes(
                root,
                &ChangeWindowAttributesAux::new().event_mask(
                    EventMask::SUBSTRUCTURE_REDIRECT
                        | EventMask::SUBSTRUCTURE_NOTIFY
                        | EventMask::PROPERTY_CHANGE
                        | EventMask::KEY_PRESS,
                ),
            )?
            .check()?;

        return Ok(Self {
            connection,
            screen_number,
            root,
            screen,
            windows: Vec::new(),
            focused_window: None,
            layout: Box::new(TilingLayout),
        });
    }

    pub fn run(&mut self) -> Result<()> {
        println!("oxwm started on display {}", self.screen_number);

        keyboard::setup_keybinds(&self.connection, self.root)?;

        loop {
            let event = self.connection.wait_for_event()?;
            println!("event: {:?}", event);
            self.handle_event(event)?;
        }
    }

    fn handle_key_action(&mut self, action: KeyAction, arg: &Arg) -> Result<()> {
        match action {
            KeyAction::Spawn => {
                if let Arg::Str(command) = arg {
                    println!("Spawning: {}", command);
                    std::process::Command::new(command).spawn()?;
                }
            }
            KeyAction::KillClient => {
                println!("Closing focused window");
                if let Some(focused) = self.focused_window {
                    match self.connection.kill_client(focused) {
                        Ok(_) => {
                            self.connection.flush()?;
                            println!("Killed window {}", focused);
                        }
                        Err(e) => {
                            println!("Failed to kill window {}: {}", focused, e);
                        }
                    }
                }
            }
            KeyAction::FocusStack => {
                if let Arg::Int(direction) = arg {
                    println!("FocusStack called with direction: {}", direction);
                    println!("Windows: {:?}", self.windows);
                    println!("Current focused: {:?}", self.focused_window);
                    self.cycle_focus(*direction)?;
                    println!("New focused: {:?}", self.focused_window);
                }
            }
            KeyAction::Quit => {
                println!("Quitting window manager");
                std::process::exit(0);
            }
            KeyAction::None => {
                //no-op
            }
        }
        Ok(())
    }

    pub fn cycle_focus(&mut self, direction: i32) -> Result<()> {
        if self.windows.is_empty() {
            return Ok(());
        }

        let next_window = if let Some(current) = self.focused_window {
            if let Some(current_index) = self.windows.iter().position(|&w| w == current) {
                let next_index = if direction > 0 {
                    (current_index + 1) % self.windows.len()
                } else {
                    if current_index == 0 {
                        self.windows.len() - 1
                    } else {
                        current_index - 1
                    }
                };
                self.windows[next_index]
            } else {
                self.windows[0]
            }
        } else {
            self.windows[0]
        };

        self.set_focus(Some(next_window))?;
        Ok(())
    }

    pub fn set_focus(&mut self, window: Option<Window>) -> Result<()> {
        println!("set_focus called with: {:?}", window);
        self.focused_window = window;

        if let Some(win) = window {
            self.connection
                .set_input_focus(InputFocus::POINTER_ROOT, win, x11rb::CURRENT_TIME)?;
            self.connection.flush()?;
        }

        self.update_focus_visuals()?;
        Ok(())
    }

    fn update_focus_visuals(&self) -> Result<()> {
        println!("Updating focus visuals for {} windows", self.windows.len());
        for &window in &self.windows {
            let (border_color, border_width) = if self.focused_window == Some(window) {
                println!("Window {} is FOCUSED (red border)", window);
                (0xff0000, 2)
            } else {
                println!("Window {} is unfocused (gray border)", window);
                (0x888888, 2)
            };

            self.connection.configure_window(
                window,
                &ConfigureWindowAux::new().border_width(border_width),
            )?;

            self.connection.change_window_attributes(
                window,
                &ChangeWindowAttributesAux::new().border_pixel(border_color),
            )?;
        }
        self.connection.flush()?;
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::MapRequest(event) => {
                self.connection.map_window(event.window)?;
                self.windows.push(event.window);
                self.apply_layout()?;
                self.set_focus(Some(event.window))?;
            }
            Event::UnmapNotify(event) => {
                if self.windows.contains(&event.window) {
                    println!("Window {} unmapped, removing from layout", event.window);
                    self.remove_window(event.window)?;
                }
            }
            Event::DestroyNotify(event) => {
                if self.windows.contains(&event.window) {
                    println!("Window {} destroyed, removing from layout", event.window);
                    self.remove_window(event.window)?;
                }
            }
            Event::KeyPress(event) => {
                let (action, arg) = keyboard::handle_key_press(event)?;
                self.handle_key_action(action, arg)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn apply_layout(&self) -> Result<()> {
        let screen_width = self.screen.width_in_pixels as u32;
        let screen_height = self.screen.height_in_pixels as u32;
        let border_width = 2u32;

        let geometries = self
            .layout
            .arrange(&self.windows, screen_width, screen_height);

        for (window, geometry) in self.windows.iter().zip(geometries.iter()) {
            // Adjust for full borders
            let adjusted_width = geometry.width.saturating_sub(2 * border_width);
            let adjusted_height = geometry.height.saturating_sub(2 * border_width);

            self.connection.configure_window(
                *window,
                &ConfigureWindowAux::new()
                    .x(geometry.x_coordinate)
                    .y(geometry.y_coordinate)
                    .width(adjusted_width)
                    .height(adjusted_height),
            )?;
        }
        self.connection.flush()?;
        return Ok(());
    }

    fn remove_window(&mut self, window: Window) -> Result<()> {
        let initial_count = self.windows.len();
        self.windows.retain(|&w| w != window);

        if self.windows.len() < initial_count {
            println!("Removed window {} from management", window);

            if self.focused_window == Some(window) {
                let new_focus = if self.windows.is_empty() {
                    None
                } else {
                    Some(self.windows[self.windows.len() - 1])
                };
                self.set_focus(new_focus)?;
            }

            self.apply_layout()?;
        }

        Ok(())
    }
}
