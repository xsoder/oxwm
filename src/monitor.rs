use crate::errors::WmError;
use x11rb::protocol::randr::{self, ConnectionExt as _};
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
    root: Window,
) -> WmResult<Vec<Monitor>> {
    let randr_version = match connection.randr_query_version(1, 2) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply.major_version >= 1 && reply.minor_version >= 2,
            Err(_) => false,
        },
        Err(_) => false,
    };

    if !randr_version {
        eprintln!("RandR 1.2+ not available, using single monitor");
        return Ok(vec![Monitor::new(
            0,
            0,
            screen.width_in_pixels as u32,
            screen.height_in_pixels as u32,
        )]);
    }

    let resources = match connection.randr_get_screen_resources(root) {
        Ok(cookie) => match cookie.reply() {
            Ok(res) => res,
            Err(_) => {
                eprintln!("Failed to get screen resources, using single monitor");
                return Ok(vec![Monitor::new(
                    0,
                    0,
                    screen.width_in_pixels as u32,
                    screen.height_in_pixels as u32,
                )]);
            }
        },
        Err(_) => {
            eprintln!("Failed to query screen resources, using single monitor");
            return Ok(vec![Monitor::new(
                0,
                0,
                screen.width_in_pixels as u32,
                screen.height_in_pixels as u32,
            )]);
        }
    };

    let mut monitors = Vec::new();

    for &output in &resources.outputs {
        let output_info = match connection.randr_get_output_info(output, 0) {
            Ok(cookie) => match cookie.reply() {
                Ok(info) => info,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if output_info.connection != randr::Connection::CONNECTED {
            continue;
        }

        if output_info.crtc == 0 {
            continue;
        }

        let crtc_info = match connection.randr_get_crtc_info(output_info.crtc, 0) {
            Ok(cookie) => match cookie.reply() {
                Ok(info) => info,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if crtc_info.width == 0 || crtc_info.height == 0 {
            continue;
        }

        monitors.push(Monitor::new(
            crtc_info.x as i32,
            crtc_info.y as i32,
            crtc_info.width as u32,
            crtc_info.height as u32,
        ));
    }

    if monitors.is_empty() {
        eprintln!("No monitors detected via RandR, using full screen");
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
