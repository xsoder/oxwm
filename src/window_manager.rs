use crate::keys;
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
            layout: Box::new(TilingLayout),
        });
    }

    pub fn run(&mut self) -> Result<()> {
        println!("oxwm started on display {}", self.screen_number);

        keys::setup_keybinds(&self.connection, self.root)?;

        loop {
            let event = self.connection.wait_for_event()?;
            println!("event: {:?}", event);
            self.handle_event(event)?;
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::MapRequest(event) => {
                self.connection.map_window(event.window)?;
                self.windows.push(event.window);
                self.apply_layout()?;
                self.connection.set_input_focus(
                    InputFocus::POINTER_ROOT,
                    event.window,
                    x11rb::CURRENT_TIME,
                )?;
                self.connection.flush()?;
            }
            Event::KeyPress(event) => {
                println!("KeyPress event received!");
                keys::handle_key_press(&self.connection, event)?;
            }
            _ => {}
        }
        return Ok(());
    }

    fn apply_layout(&self) -> Result<()> {
        let screen_width = self.screen.width_in_pixels as u32;
        let screen_height = self.screen.height_in_pixels as u32;

        let geometries = self
            .layout
            .arrange(&self.windows, screen_width, screen_height);

        for (window, geometry) in self.windows.iter().zip(geometries.iter()) {
            self.connection.configure_window(
                *window,
                &ConfigureWindowAux::new()
                    .x(geometry.x_coordinate)
                    .y(geometry.y_coordinate)
                    .width(geometry.width)
                    .height(geometry.height)
                    .border_width(1),
            )?;
        }
        return Ok(());
    }
}
