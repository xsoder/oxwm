use super::{GapConfig, Layout, WindowGeometry};
use x11rb::protocol::xproto::Window;

pub struct TabbedLayout;

pub const TAB_BAR_HEIGHT: u32 = 28;

impl Layout for TabbedLayout {
    fn name(&self) -> &'static str {
        super::LayoutType::Tabbed.as_str()
    }

    fn symbol(&self) -> &'static str {
        "[=]"
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

        let x = gaps.outer_horizontal as i32;
        let y = (gaps.outer_vertical + TAB_BAR_HEIGHT) as i32;
        let width = screen_width.saturating_sub(2 * gaps.outer_horizontal);
        let height = screen_height
            .saturating_sub(2 * gaps.outer_vertical)
            .saturating_sub(TAB_BAR_HEIGHT);

        let geometry = WindowGeometry {
            x_coordinate: x,
            y_coordinate: y,
            width,
            height,
        };

        vec![geometry; window_count]
    }
}
