use super::{Layout, WindowGeometry};
use x11rb::protocol::xproto::Window;

pub struct TilingLayout;

impl Layout for TilingLayout {
    fn arrange(
        &self,
        windows: &[Window],
        screen_width: u32,
        screen_height: u32,
    ) -> Vec<WindowGeometry> {
        let window_count = windows.len();
        if window_count == 0 {
            return Vec::new();
        }

        if window_count == 1 {
            vec![WindowGeometry {
                x_coordinate: 0,
                y_coordinate: 0,
                width: screen_width,
                height: screen_height,
            }]
        } else {
            let master_width = screen_width / 2;
            let mut geometries = vec![WindowGeometry {
                x_coordinate: 0,
                y_coordinate: 0,
                width: master_width,
                height: screen_height,
            }];

            let stack_height = screen_height / (window_count - 1) as u32;
            for i in 1..window_count {
                let y_offset = ((i - 1) as u32) * stack_height;
                geometries.push(WindowGeometry {
                    x_coordinate: master_width as i32,
                    y_coordinate: y_offset as i32,
                    width: master_width,
                    height: stack_height,
                });
            }
            return geometries;
        }
    }
}
