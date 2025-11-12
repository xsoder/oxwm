use crate::bar::font::{Font, FontDraw};
use crate::errors::X11Error;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

pub mod error;
pub mod keybind;

pub use error::ErrorOverlay;
pub use keybind::KeybindOverlay;

pub trait Overlay {
    fn window(&self) -> Window;
    fn is_visible(&self) -> bool;
    fn hide(&mut self, connection: &RustConnection) -> Result<(), X11Error>;
    fn draw(&self, connection: &RustConnection, font: &Font) -> Result<(), X11Error>;
}

pub struct OverlayBase {
    pub window: Window,
    pub width: u16,
    pub height: u16,
    pub graphics_context: Gcontext,
    pub font_draw: FontDraw,
    pub is_visible: bool,
    pub background_color: u32,
    pub foreground_color: u32,
}

impl OverlayBase {
    pub fn new(
        connection: &RustConnection,
        screen: &Screen,
        screen_num: usize,
        display: *mut x11::xlib::Display,
        width: u16,
        height: u16,
        border_width: u16,
        border_color: u32,
        background_color: u32,
        foreground_color: u32,
    ) -> Result<Self, X11Error> {
        let window = connection.generate_id()?;
        let graphics_context = connection.generate_id()?;

        connection.create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            screen.root,
            0,
            0,
            width,
            height,
            border_width,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &CreateWindowAux::new()
                .background_pixel(background_color)
                .border_pixel(border_color)
                .event_mask(EventMask::EXPOSURE | EventMask::BUTTON_PRESS | EventMask::KEY_PRESS)
                .override_redirect(1),
        )?;

        connection.create_gc(
            graphics_context,
            window,
            &CreateGCAux::new()
                .foreground(foreground_color)
                .background(background_color),
        )?;

        connection.flush()?;

        let visual = unsafe { x11::xlib::XDefaultVisual(display, screen_num as i32) };
        let colormap = unsafe { x11::xlib::XDefaultColormap(display, screen_num as i32) };

        let font_draw = FontDraw::new(display, window as x11::xlib::Drawable, visual, colormap)?;

        Ok(OverlayBase {
            window,
            width,
            height,
            graphics_context,
            font_draw,
            is_visible: false,
            background_color,
            foreground_color,
        })
    }

    pub fn configure(
        &mut self,
        connection: &RustConnection,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> Result<(), X11Error> {
        self.width = width;
        self.height = height;

        connection.configure_window(
            self.window,
            &ConfigureWindowAux::new()
                .x(x as i32)
                .y(y as i32)
                .width(width as u32)
                .height(height as u32),
        )?;

        Ok(())
    }

    pub fn show(&mut self, connection: &RustConnection) -> Result<(), X11Error> {
        connection.configure_window(
            self.window,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        )?;

        connection.map_window(self.window)?;
        connection.flush()?;

        self.is_visible = true;

        Ok(())
    }

    pub fn hide(&mut self, connection: &RustConnection) -> Result<(), X11Error> {
        if self.is_visible {
            connection.unmap_window(self.window)?;
            connection.flush()?;
            self.is_visible = false;
        }
        Ok(())
    }

    pub fn draw_background(&self, connection: &RustConnection) -> Result<(), X11Error> {
        connection.change_gc(
            self.graphics_context,
            &ChangeGCAux::new().foreground(self.background_color),
        )?;
        connection.poly_fill_rectangle(
            self.window,
            self.graphics_context,
            &[Rectangle {
                x: 0,
                y: 0,
                width: self.width,
                height: self.height,
            }],
        )?;
        Ok(())
    }
}
