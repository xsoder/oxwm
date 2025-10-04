use crate::bar::Bar;
use crate::config::{BORDER_FOCUSED, BORDER_UNFOCUSED, BORDER_WIDTH, TAG_COUNT};
use crate::keyboard::{self, Arg, KeyAction};
use crate::layout::Layout;
use crate::layout::tiling::TilingLayout;
use anyhow::Result;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

pub type TagMask = u32;
pub fn tag_mask(tag: usize) -> TagMask {
    1 << tag
}

pub struct WindowManager {
    connection: RustConnection,
    screen_number: usize,
    root: Window,
    screen: Screen,
    windows: Vec<Window>,
    focused_window: Option<Window>,
    layout: Box<dyn Layout>,
    window_tags: std::collections::HashMap<Window, TagMask>,
    selected_tags: TagMask,
    bar: Bar,
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

        let bar = Bar::new(&connection, &screen, screen_number)?;

        let mut window_manger = Self {
            connection,
            screen_number,
            root,
            screen,
            windows: Vec::new(),
            focused_window: None,
            layout: Box::new(TilingLayout),
            window_tags: std::collections::HashMap::new(),
            selected_tags: tag_mask(0),
            bar,
        };

        window_manger.scan_existing_windows()?;

        Ok(window_manger)
    }

    fn scan_existing_windows(&mut self) -> Result<()> {
        let tree = self.connection.query_tree(self.root)?.reply()?;

        for &window in &tree.children {
            if let Ok(attrs) = self.connection.get_window_attributes(window)?.reply() {
                if window != self.bar.window()
                    && attrs.map_state == MapState::VIEWABLE
                    && !attrs.override_redirect
                {
                    self.windows.push(window);
                    self.window_tags.insert(window, self.selected_tags);
                }
            }
        }
        if let Some(&first) = self.windows.first() {
            self.set_focus(Some(first))?;
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<bool> {
        println!("oxwm started on display {}", self.screen_number);

        keyboard::setup_keybinds(&self.connection, self.root)?;
        self.update_bar()?;

        loop {
            self.bar.update_blocks()?;

            if let Ok(Some(event)) = self.connection.poll_for_event() {
                if let Some(should_restart) = self.handle_event(event)? {
                    return Ok(should_restart);
                }
            }

            if self.bar.needs_redraw() {
                self.update_bar()?;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn update_bar(&mut self) -> Result<()> {
        let mut occupied_tags: TagMask = 0;
        for &tags in self.window_tags.values() {
            occupied_tags |= tags;
        }

        self.bar.invalidate();
        self.bar
            .draw(&self.connection, self.selected_tags, occupied_tags)?;
        Ok(())
    }

    fn handle_key_action(&mut self, action: KeyAction, arg: &Arg) -> Result<()> {
        match action {
            KeyAction::Spawn => match arg {
                Arg::Str(command) => {
                    std::process::Command::new(command).spawn()?;
                }
                Arg::Array(cmd) => {
                    if let Some((program, args)) = cmd.split_first() {
                        std::process::Command::new(program).args(args).spawn()?;
                    }
                }
                _ => {}
            },
            KeyAction::KillClient => {
                if let Some(focused) = self.focused_window {
                    match self.connection.kill_client(focused) {
                        Ok(_) => {
                            self.connection.flush()?;
                        }
                        Err(e) => {
                            eprintln!("Failed to kill window {}: {}", focused, e);
                        }
                    }
                }
            }
            KeyAction::FocusStack => {
                if let Arg::Int(direction) = arg {
                    self.cycle_focus(*direction)?;
                }
            }
            KeyAction::Quit | KeyAction::Restart => {
                //no-op
            }
            KeyAction::ViewTag => {
                if let Arg::Int(tag_index) = arg {
                    self.view_tag(*tag_index as usize)?;
                }
            }
            KeyAction::MoveToTag => {
                if let Arg::Int(tag_index) = arg {
                    self.move_to_tag(*tag_index as usize)?;
                }
            }
            KeyAction::None => {
                //no-op
            }
        }
        Ok(())
    }

    fn is_window_visible(&self, window: Window) -> bool {
        if let Some(&tags) = self.window_tags.get(&window) {
            (tags & self.selected_tags) != 0
        } else {
            false
        }
    }

    fn visible_windows(&self) -> Vec<Window> {
        self.windows
            .iter()
            .filter(|&&w| self.is_window_visible(w))
            .copied()
            .collect()
    }

    fn update_window_visibility(&self) -> Result<()> {
        for &window in &self.windows {
            if self.is_window_visible(window) {
                self.connection.map_window(window)?;
            } else {
                self.connection.unmap_window(window)?;
            }
        }
        self.connection.flush()?;
        Ok(())
    }

    pub fn view_tag(&mut self, tag_index: usize) -> Result<()> {
        if tag_index >= TAG_COUNT {
            return Ok(());
        }

        self.selected_tags = tag_mask(tag_index);
        self.update_window_visibility()?;
        self.apply_layout()?;
        self.update_bar()?;

        let visible = self.visible_windows();
        self.set_focus(visible.first().copied())?;

        Ok(())
    }

    pub fn move_to_tag(&mut self, tag_index: usize) -> Result<()> {
        if tag_index >= TAG_COUNT {
            return Ok(());
        }

        if let Some(focused) = self.focused_window {
            let mask = tag_mask(tag_index);
            self.window_tags.insert(focused, mask);
            self.update_window_visibility()?;
            self.apply_layout()?;
            self.update_bar()?;
        }

        Ok(())
    }

    pub fn cycle_focus(&mut self, direction: i32) -> Result<()> {
        let visible = self.visible_windows();

        if visible.is_empty() {
            return Ok(());
        }

        let next_window = if let Some(current) = self.focused_window {
            if let Some(current_index) = visible.iter().position(|&w| w == current) {
                let next_index = if direction > 0 {
                    (current_index + 1) % visible.len()
                } else {
                    (current_index + visible.len() - 1) % visible.len()
                };
                visible[next_index]
            } else {
                visible[0]
            }
        } else {
            visible[0]
        };

        self.set_focus(Some(next_window))?;
        Ok(())
    }

    pub fn set_focus(&mut self, window: Option<Window>) -> Result<()> {
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
        for &window in &self.windows {
            let (border_color, border_width) = if self.focused_window == Some(window) {
                (BORDER_FOCUSED, BORDER_WIDTH)
            } else {
                (BORDER_UNFOCUSED, BORDER_WIDTH)
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

    fn handle_event(&mut self, event: Event) -> Result<Option<bool>> {
        match event {
            Event::MapRequest(event) => {
                self.connection.map_window(event.window)?;
                self.windows.push(event.window);
                self.window_tags.insert(event.window, self.selected_tags);
                self.apply_layout()?;
                self.update_bar()?;
                self.set_focus(Some(event.window))?;
            }
            Event::UnmapNotify(event) => {
                if self.windows.contains(&event.window) && self.is_window_visible(event.window) {
                    self.remove_window(event.window)?;
                }
            }
            Event::DestroyNotify(event) => {
                if self.windows.contains(&event.window) {
                    self.remove_window(event.window)?;
                }
            }
            Event::KeyPress(event) => {
                let (action, arg) = keyboard::handle_key_press(event)?;

                match action {
                    KeyAction::Quit => return Ok(Some(false)),
                    KeyAction::Restart => return Ok(Some(true)),
                    _ => self.handle_key_action(action, arg)?,
                }
            }
            Event::ButtonPress(event) => {
                if event.event == self.bar.window() {
                    if let Some(tag_index) = self.bar.handle_click(event.event_x) {
                        self.view_tag(tag_index)?;
                    }
                }
            }
            Event::Expose(event) => {
                if event.window == self.bar.window() {
                    self.bar.invalidate();
                    self.update_bar()?;
                }
            }
            _ => {}
        }
        Ok(None)
    }
    fn apply_layout(&self) -> Result<()> {
        let screen_width = self.screen.width_in_pixels as u32;
        let screen_height = self.screen.height_in_pixels as u32;
        let border_width = BORDER_WIDTH;

        let bar_height = self.bar.height() as u32;
        let usable_height = screen_height.saturating_sub(bar_height);

        let visible = self.visible_windows();
        let geometries = self.layout.arrange(&visible, screen_width, usable_height);

        for (window, geometry) in visible.iter().zip(geometries.iter()) {
            let adjusted_width = geometry.width.saturating_sub(2 * border_width);
            let adjusted_height = geometry.height.saturating_sub(2 * border_width);

            let adjusted_y = geometry.y_coordinate + bar_height as i32;

            self.connection.configure_window(
                *window,
                &ConfigureWindowAux::new()
                    .x(geometry.x_coordinate)
                    .y(adjusted_y)
                    .width(adjusted_width)
                    .height(adjusted_height),
            )?;
        }
        self.connection.flush()?;
        Ok(())
    }

    fn remove_window(&mut self, window: Window) -> Result<()> {
        let initial_count = self.windows.len();
        self.windows.retain(|&w| w != window);
        self.window_tags.remove(&window);

        if self.windows.len() < initial_count {
            if self.focused_window == Some(window) {
                let new_focus = if self.windows.is_empty() {
                    None
                } else {
                    Some(self.windows[self.windows.len() - 1])
                };
                self.set_focus(new_focus)?;
            }

            self.apply_layout()?;
            self.update_bar()?;
        }
        Ok(())
    }
}
