pub mod tiling;

use x11rb::protocol::xproto::Window;

pub trait Layout {
    fn arrange(
        &self,
        windows: &[Window],
        screen_width: u32,
        screen_height: u32,
    ) -> Vec<WindowGeometry>;
}

pub struct WindowGeometry {
    pub x_coordinate: i32,
    pub y_coordinate: i32,
    pub width: u32,
    pub height: u32,
}
