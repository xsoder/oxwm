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
    let fallback_monitors = || {
        vec![Monitor::new(
            0,
            0,
            screen.width_in_pixels as u32,
            screen.height_in_pixels as u32,
        )]
    };

    let mut monitors = Vec::<Monitor>::new();

    let xinerama_active = connection
        .xinerama_is_active()
        .ok()
        .and_then(|cookie| cookie.reply().ok())
        .map_or(false, |reply| reply.state != 0);

    if xinerama_active {
        let Ok(xinerama_cookie) = connection.xinerama_query_screens() else {
            return Ok(fallback_monitors());
        };
        let Ok(xinerama_reply) = xinerama_cookie.reply() else {
            return Ok(fallback_monitors());
        };

        for screen_info in &xinerama_reply.screen_info {
            let has_valid_dimensions = screen_info.width > 0 && screen_info.height > 0;
            if !has_valid_dimensions {
                continue;
            }

            let x_position = screen_info.x_org as i32;
            let y_position = screen_info.y_org as i32;
            let width_in_pixels = screen_info.width as u32;
            let height_in_pixels = screen_info.height as u32;

            let is_duplicate_monitor = monitors.iter().any(|monitor| {
                monitor.x == x_position
                    && monitor.y == y_position
                    && monitor.width == width_in_pixels
                    && monitor.height == height_in_pixels
            });

            if !is_duplicate_monitor {
                monitors.push(Monitor::new(
                    x_position,
                    y_position,
                    width_in_pixels,
                    height_in_pixels,
                ));
            }
        }
    }

    if monitors.is_empty() {
        monitors = fallback_monitors();
    }

    monitors.sort_by(|a, b| match a.y.cmp(&b.y) {
        std::cmp::Ordering::Equal => a.x.cmp(&b.x),
        other => other,
    });

    Ok(monitors)
}
