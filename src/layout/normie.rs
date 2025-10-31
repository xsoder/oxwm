use super::{GapConfig, Layout, WindowGeometry};
use x11rb::protocol::xproto::Window;

pub struct NormieLayout;

// This layout should return a no-op similar to DWM.C's "null" mode.
impl Layout for NormieLayout {
    fn name(&self) -> &'static str {
        super::LayoutType::Normie.as_str()
    }

    fn symbol(&self) -> &'static str {
        "><>"
    }

    fn arrange(
        &self,
        windows: &[Window],
        screen_width: u32,
        screen_height: u32,
        _gaps: &GapConfig,
    ) -> Vec<WindowGeometry> {
        const DEFAULT_WIDTH_RATIO: f32 = 0.6;
        const DEFAULT_HEIGHT_RATIO: f32 = 0.6;

        windows
            .iter()
            .map(|_| {
                let width = ((screen_width as f32) * DEFAULT_WIDTH_RATIO) as u32;
                let height = ((screen_height as f32) * DEFAULT_HEIGHT_RATIO) as u32;

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
