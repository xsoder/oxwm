use super::font::{Font, FontDraw};
use crate::config::{BAR_HEIGHT, FONT, SCHEME_NORMAL, SCHEME_OCCUPIED, SCHEME_SELECTED, TAGS};
use anyhow::Result;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

pub struct Bar {
    window: Window,
    width: u16,
    height: u16,
    graphics_context: Gcontext,

    font: Font,
    font_draw: FontDraw,
    display: *mut x11::xlib::Display,

    tag_widths: Vec<u16>,
    needs_redraw: bool,
}

impl Bar {
    pub fn new(connection: &RustConnection, screen: &Screen, screen_num: usize) -> Result<Self> {
        let window = connection.generate_id()?;
        let graphics_context = connection.generate_id()?;

        let width = screen.width_in_pixels;
        let height = BAR_HEIGHT;

        connection.create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            screen.root,
            0,
            0,
            width,
            height,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &CreateWindowAux::new()
                .background_pixel(SCHEME_NORMAL.background)
                .event_mask(EventMask::EXPOSURE | EventMask::BUTTON_PRESS)
                .override_redirect(1),
        )?;

        connection.create_gc(
            graphics_context,
            window,
            &CreateGCAux::new()
                .foreground(SCHEME_NORMAL.foreground)
                .background(SCHEME_NORMAL.background),
        )?;

        connection.map_window(window)?;
        connection.flush()?;

        let display = unsafe { x11::xlib::XOpenDisplay(std::ptr::null()) };
        if display.is_null() {
            anyhow::bail!("Failed to open X11 display for XFT");
        }

        let font = Font::new(display, screen_num as i32, FONT)?;

        let visual = unsafe { x11::xlib::XDefaultVisual(display, screen_num as i32) };
        let colormap = unsafe { x11::xlib::XDefaultColormap(display, screen_num as i32) };

        let font_draw = FontDraw::new(display, window as x11::xlib::Drawable, visual, colormap)?;

        let tag_widths = TAGS.iter().map(|tag| font.text_width(tag) + 10).collect();

        Ok(Bar {
            window,
            width,
            height,
            graphics_context,
            font,
            font_draw,
            display,
            tag_widths,
            needs_redraw: true,
        })
    }

    pub fn window(&self) -> Window {
        self.window
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn invalidate(&mut self) {
        self.needs_redraw = true;
    }

    pub fn draw(
        &mut self,
        connection: &RustConnection,
        current_tags: u32,
        occupied_tags: u32,
    ) -> Result<()> {
        if !self.needs_redraw {
            return Ok(());
        }
        connection.change_gc(
            self.graphics_context,
            &ChangeGCAux::new().foreground(SCHEME_NORMAL.background),
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

        let mut x_position: i16 = 0;

        for (tag_index, tag) in TAGS.iter().enumerate() {
            let tag_mask = 1 << tag_index;
            let is_selected = (current_tags & tag_mask) != 0;
            let is_occupied = (occupied_tags & tag_mask) != 0;

            let tag_width = self.tag_widths[tag_index];

            let scheme = if is_selected {
                &SCHEME_SELECTED
            } else if is_occupied {
                &SCHEME_OCCUPIED
            } else {
                &SCHEME_NORMAL
            };

            if is_selected {
                connection.change_gc(
                    self.graphics_context,
                    &ChangeGCAux::new().foreground(scheme.background),
                )?;
                connection.poly_fill_rectangle(
                    self.window,
                    self.graphics_context,
                    &[Rectangle {
                        x: x_position,
                        y: 0,
                        width: tag_width,
                        height: self.height,
                    }],
                )?;
            }

            let text_y = (self.height as i16 / 2) + (self.font.ascent() / 2);
            self.font_draw
                .draw_text(&self.font, scheme.foreground, x_position + 5, text_y, tag);

            if is_selected {
                let underline_height = 2;
                connection.change_gc(
                    self.graphics_context,
                    &ChangeGCAux::new().foreground(scheme.border),
                )?;
                connection.poly_fill_rectangle(
                    self.window,
                    self.graphics_context,
                    &[Rectangle {
                        x: x_position,
                        y: (self.height - underline_height) as i16,
                        width: tag_width,
                        height: underline_height,
                    }],
                )?;
            }

            x_position += tag_width as i16;
        }

        connection.flush()?;

        unsafe {
            x11::xlib::XFlush(self.display);
        }

        self.needs_redraw = false;

        Ok(())
    }

    pub fn handle_click(&self, click_x: i16) -> Option<usize> {
        let mut current_x_position = 0;

        for (tag_index, &tag_width) in self.tag_widths.iter().enumerate() {
            if click_x >= current_x_position && click_x < current_x_position + tag_width as i16 {
                return Some(tag_index);
            }
            current_x_position += tag_width as i16;
        }
        None
    }
}
