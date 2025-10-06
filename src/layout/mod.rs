pub mod tiling;

use x11rb::protocol::xproto::Window;

pub struct GapConfig {
    pub inner_horizontal: u32,
    pub inner_vertical: u32,
    pub outer_horizontal: u32,
    pub outer_vertical: u32,
}

pub trait Layout {
    fn arrange(
        &self,
        windows: &[Window],
        screen_width: u32,
        screen_height: u32,
        gaps: &GapConfig,
    ) -> Vec<WindowGeometry>;
}

pub struct WindowGeometry {
    pub x_coordinate: i32,
    pub y_coordinate: i32,
    pub width: u32,
    pub height: u32,
}
