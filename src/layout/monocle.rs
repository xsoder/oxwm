use super::{GapConfig, Layout, WindowGeometry};
use x11rb::protocol::xproto::Window;

pub struct MonocleLayout;

impl Layout for MonocleLayout {
    fn name(&self) -> &'static str {
        super::LayoutType::Monocle.as_str()
    }

    fn symbol(&self) -> &'static str {
        "[M]"
    }

    fn arrange(
        &self,
        windows: &[Window],
        screen_width: u32,
        screen_height: u32,
        gaps: &GapConfig,
        _master_factor: f32,
        _num_master: i32,
    ) -> Vec<WindowGeometry> {
        let window_count = windows.len();
        if window_count == 0 {
            return Vec::new();
        }

        let x = gaps.outer_horizontal as i32;
        let y = gaps.outer_vertical as i32;
        let width = screen_width.saturating_sub(2 * gaps.outer_horizontal);
        let height = screen_height.saturating_sub(2 * gaps.outer_vertical);

        let geometry = WindowGeometry {
            x_coordinate: x,
            y_coordinate: y,
            width,
            height,
        };

        vec![geometry; window_count]
    }
}
