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
        _windows: &[Window],
        _screen_width: u32,
        _screen_height: u32,
        _gaps: &GapConfig,
        _master_factor: f32,
        _num_master: i32,
        _smartgaps_enabled: bool,
    ) -> Vec<WindowGeometry> {
        Vec::new()
    }
}
