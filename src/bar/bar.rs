use super::BAR_HEIGHT;
use crate::config::{SCHEME_NORMAL, SCHEME_OCCUPIED, SCHEME_SELECTED, TAGS};
use anyhow::Result;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

pub struct Bar {
    window: Window,
    width: u16,
    height: u16,
    graphics_context: Gcontext,

    tag_widths: Vec<u16>,
    needs_redraw: bool,
}

impl Bar {
    pub fn new<C: Connection>(connection: &C, screen: &Screen) -> Result<Self> {
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

        // TODO: actual text width calculation when we add fonts
        let tag_widths = TAGS.iter().map(|tag| (tag.len() as u16 * 8) + 10).collect();

        Ok(Bar {
            window,
            width,
            height,
            graphics_context,
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

    pub fn draw<C: Connection>(
        &mut self,
        connection: &C,
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
            } else if is_occupied {
                connection.change_gc(
                    self.graphics_context,
                    &ChangeGCAux::new().foreground(scheme.border),
                )?;
                connection.poly_fill_rectangle(
                    self.window,
                    self.graphics_context,
                    &[Rectangle {
                        x: x_position,
                        y: 0,
                        width: tag_width,
                        height: 2,
                    }],
                )?;
            }

            connection.change_gc(
                self.graphics_context,
                &ChangeGCAux::new().foreground(scheme.foreground),
            )?;

            // TODO: Replace with actual font rendering later
            connection.image_text8(
                self.window,
                self.graphics_context,
                x_position + 5,
                self.height as i16 - 5,
                tag.as_bytes(),
            )?;

            x_position += tag_width as i16;
        }

        connection.flush()?;
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
