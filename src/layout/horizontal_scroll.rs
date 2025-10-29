use super::{GapConfig, Layout, WindowGeometry};
use x11rb::protocol::xproto::Window;

pub struct HorizontalScrollLayout {
    pub window_width: u32,
}

impl HorizontalScrollLayout {
    pub fn new(window_width: u32) -> Self {
        Self { window_width }
    }

    pub fn total_width(&self, window_count: usize, gaps: &GapConfig) -> u32 {
        if window_count == 0 {
            return 0;
        }
        let window_spacing = self.window_width + gaps.inner_horizontal;
        (window_count as u32 * window_spacing)
            .saturating_sub(gaps.inner_horizontal)
            + (gaps.outer_horizontal * 2)
    }

    pub fn max_scroll_offset(&self, window_count: usize, screen_width: u32, gaps: &GapConfig) -> i32 {
        let total = self.total_width(window_count, gaps);
        if total <= screen_width {
            0
        } else {
            (total - screen_width) as i32
        }
    }

}

impl Layout for HorizontalScrollLayout {
    fn name(&self) -> &'static str {
        super::HORIZONTAL_SCROLL
    }

    fn symbol(&self) -> &'static str {
        "[H]"
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

        let mut geometries = Vec::new();

        let window_height = screen_height
            .saturating_sub(gaps.outer_vertical * 2);

        let effective_width = self.window_width
            .saturating_sub(gaps.outer_horizontal * 2)
            .saturating_sub(gaps.inner_horizontal);

        for i in 0..window_count {
            let x = (i as u32 * (self.window_width + gaps.inner_horizontal)) as i32
                + gaps.outer_horizontal as i32;
            let y = gaps.outer_vertical as i32;

            geometries.push(WindowGeometry {
                x_coordinate: x,
                y_coordinate: y,
                width: effective_width,
                height: window_height,
            });
        }

        geometries
    }
}
