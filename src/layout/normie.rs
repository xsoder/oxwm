use super::{GapConfig, Layout, WindowGeometry};
use x11rb::protocol::xproto::Window;

pub struct NormieLayout;

impl Layout for NormieLayout {
    fn name(&self) -> &'static str {
        super::NORMIE
    }

    fn arrange(
        &self,
        windows: &[Window],
        screen_width: u32,
        screen_height: u32,
        _gaps: &GapConfig,
    ) -> Vec<WindowGeometry> {
        // Floating layout: all windows open centered with a reasonable default size
        // This mimics dwm's NULL layout behavior where windows float freely
        const DEFAULT_WIDTH_RATIO: f32 = 0.6;
        const DEFAULT_HEIGHT_RATIO: f32 = 0.6;

        windows
            .iter()
            .map(|_| {
                // Calculate default window dimensions (60% of screen)
                let width = ((screen_width as f32) * DEFAULT_WIDTH_RATIO) as u32;
                let height = ((screen_height as f32) * DEFAULT_HEIGHT_RATIO) as u32;

                // Center the window on the screen
                let x = ((screen_width - width) / 2) as i32;
                let y = ((screen_height - height) / 2) as i32;

                WindowGeometry {
                    x_coordinate: x,
                    y_coordinate: y,
                    width,
                    height,
                }
            })
            .collect()
    }
}
