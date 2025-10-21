use crate::errors::WmError;
use x11rb::protocol::xinerama::ConnectionExt as _;
use x11rb::protocol::xproto::{Screen, Window};
use x11rb::rust_connection::RustConnection;

type WmResult<T> = Result<T, WmError>;

#[derive(Debug, Clone)]
pub struct Monitor {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub selected_tags: u32,
    pub focused_window: Option<Window>,
}

impl Monitor {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            selected_tags: 1,
            focused_window: None,
        }
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x + self.width as i32
            && y >= self.y
            && y < self.y + self.height as i32
    }
}

pub fn detect_monitors(
    connection: &RustConnection,
    screen: &Screen,
    _root: Window,
) -> WmResult<Vec<Monitor>> {
    let mut monitors = Vec::new();

    if let Ok(cookie) = connection.xinerama_is_active() {
        if let Ok(reply) = cookie.reply() {
            if reply.state != 0 {
                if let Ok(screens_cookie) = connection.xinerama_query_screens() {
                    if let Ok(screens_reply) = screens_cookie.reply() {
                        for screen_info in &screens_reply.screen_info {
                            if screen_info.width == 0 || screen_info.height == 0 {
                                continue;
                            }

                            let is_unique = !monitors.iter().any(|m: &Monitor| {
                                m.x == screen_info.x_org as i32
                                    && m.y == screen_info.y_org as i32
                                    && m.width == screen_info.width as u32
                                    && m.height == screen_info.height as u32
                            });

                            if is_unique {
                                monitors.push(Monitor::new(
                                    screen_info.x_org as i32,
                                    screen_info.y_org as i32,
                                    screen_info.width as u32,
                                    screen_info.height as u32,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if monitors.is_empty() {
        monitors.push(Monitor::new(
            0,
            0,
            screen.width_in_pixels as u32,
            screen.height_in_pixels as u32,
        ));
    }

    monitors.sort_by(|a, b| {
        let y_cmp = a.y.cmp(&b.y);
        if y_cmp == std::cmp::Ordering::Equal {
            a.x.cmp(&b.x)
        } else {
            y_cmp
        }
    });

    Ok(monitors)
}
