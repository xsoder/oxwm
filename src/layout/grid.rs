use super::{GapConfig, Layout, WindowGeometry};
use x11rb::protocol::xproto::Window;

pub struct GridLayout;

impl Layout for GridLayout {
    fn name(&self) -> &'static str {
        super::LayoutType::Grid.as_str()
    }

    fn symbol(&self) -> &'static str {
        "[#]"
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

            return vec![WindowGeometry {
                x_coordinate: x,
                y_coordinate: y,
                width,
                height,
            }];
        }

        let cols = (window_count as f64).sqrt().ceil() as usize;
        let rows = (window_count as f64 / cols as f64).ceil() as usize;

        let mut geometries = Vec::new();

        let total_horizontal_gaps =
            gaps.outer_horizontal * 2 + gaps.inner_horizontal * (cols as u32 - 1);
        let total_vertical_gaps = gaps.outer_vertical * 2 + gaps.inner_vertical * (rows as u32 - 1);

        let cell_width = screen_width.saturating_sub(total_horizontal_gaps) / cols as u32;
        let cell_height = screen_height.saturating_sub(total_vertical_gaps) / rows as u32;

        for (index, _window) in windows.iter().enumerate() {
            let row = index / cols;
            let col = index % cols;

            let is_last_row = row == rows - 1;
            let windows_in_last_row = window_count - (rows - 1) * cols;

            let (x, y, width, height) = if is_last_row && windows_in_last_row < cols {
                let last_row_col = index % cols;
                let last_row_cell_width =
                    screen_width.saturating_sub(total_horizontal_gaps.saturating_sub(
                        gaps.inner_horizontal * (cols as u32 - windows_in_last_row as u32),
                    )) / windows_in_last_row as u32;

                let x = gaps.outer_horizontal
                    + last_row_col as u32 * (last_row_cell_width + gaps.inner_horizontal);
                let y = gaps.outer_vertical + row as u32 * (cell_height + gaps.inner_vertical);

                (x as i32, y as i32, last_row_cell_width, cell_height)
            } else {
                let x = gaps.outer_horizontal + col as u32 * (cell_width + gaps.inner_horizontal);
                let y = gaps.outer_vertical + row as u32 * (cell_height + gaps.inner_vertical);

                (x as i32, y as i32, cell_width, cell_height)
            };

            geometries.push(WindowGeometry {
                x_coordinate: x,
                y_coordinate: y,
                width,
                height,
            });
        }

        geometries
    }
}
