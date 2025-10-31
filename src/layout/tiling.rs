use super::{GapConfig, Layout, WindowGeometry};
use x11rb::protocol::xproto::Window;

pub struct TilingLayout;

impl Layout for TilingLayout {
    fn name(&self) -> &'static str {
        super::LayoutType::Tiling.as_str()
    }

    fn symbol(&self) -> &'static str {
        "[]="
    }

    fn arrange(
        &self,
        windows: &[Window],
        screen_width: u32,
        screen_height: u32,
        gaps: &GapConfig,
    ) -> Vec<WindowGeometry> {
        let window_count = windows.len();
        if window_count == 0 {
            return Vec::new();
        }

        if window_count == 1 {
            let x = gaps.outer_horizontal as i32;
            let y = gaps.outer_vertical as i32;
            let width = screen_width.saturating_sub(2 * gaps.outer_horizontal);
            let height = screen_height.saturating_sub(2 * gaps.outer_vertical);

            vec![WindowGeometry {
                x_coordinate: x,
                y_coordinate: y,
                width,
                height,
            }]
        } else {
            let mut geometries = Vec::new();

            let master_width = (screen_width / 2)
                .saturating_sub(gaps.outer_horizontal)
                .saturating_sub(gaps.inner_horizontal / 2);

            let master_x = gaps.outer_horizontal as i32;
            let master_y = gaps.outer_vertical as i32;
            let master_height = screen_height.saturating_sub(2 * gaps.outer_vertical);

            geometries.push(WindowGeometry {
                x_coordinate: master_x,
                y_coordinate: master_y,
                width: master_width,
                height: master_height,
            });

            let stack_count = window_count - 1;
            let stack_x = (screen_width / 2 + gaps.inner_horizontal / 2) as i32;
            let stack_width = (screen_width / 2)
                .saturating_sub(gaps.outer_horizontal)
                .saturating_sub(gaps.inner_horizontal / 2);

            let total_stack_height = screen_height.saturating_sub(2 * gaps.outer_vertical);

            let total_inner_gaps = gaps.inner_vertical * (stack_count as u32 - 1);
            let stack_height =
                total_stack_height.saturating_sub(total_inner_gaps) / stack_count as u32;

            for i in 1..window_count {
                let stack_index = i - 1;
                let y_offset = gaps.outer_vertical
                    + (stack_index as u32) * (stack_height + gaps.inner_vertical);

                geometries.push(WindowGeometry {
                    x_coordinate: stack_x,
                    y_coordinate: y_offset as i32,
                    width: stack_width,
                    height: stack_height,
                });
            }

            return geometries;
        }
    }
}
