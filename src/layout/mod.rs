pub mod normie;
pub mod tiling;
pub mod horizontal_scroll;

use x11rb::protocol::xproto::Window;

pub type LayoutBox = Box<dyn Layout>;

pub struct GapConfig {
    pub inner_horizontal: u32,
    pub inner_vertical: u32,
    pub outer_horizontal: u32,
    pub outer_vertical: u32,
}

pub const TILING: &str = "tiling";
pub const NORMIE: &str = "normie";
pub const FLOATING: &str = "floating";
pub const HORIZONTAL_SCROLL: &str = "horizontal_scroll";

pub fn layout_from_str(s: &str) -> Result<LayoutBox, String> {
    match s.to_lowercase().as_str() {
        TILING => Ok(Box::new(tiling::TilingLayout)),
        NORMIE | FLOATING => Ok(Box::new(normie::NormieLayout)),
        HORIZONTAL_SCROLL => Ok(Box::new(horizontal_scroll::HorizontalScrollLayout::new(800))),
        _ => Err(format!("Unknown layout: {}", s)),
    }
}

pub fn next_layout(current_name: &str) -> &'static str {
    match current_name {
        TILING => NORMIE,
        NORMIE => HORIZONTAL_SCROLL,
        HORIZONTAL_SCROLL => TILING,
        _ => TILING,
    }
}

pub trait Layout {
    fn arrange(
        &self,
        windows: &[Window],
        screen_width: u32,
        screen_height: u32,
        gaps: &GapConfig,
    ) -> Vec<WindowGeometry>;
    fn name(&self) -> &'static str;
    fn symbol(&self) -> &'static str;
}

pub struct WindowGeometry {
    pub x_coordinate: i32,
    pub y_coordinate: i32,
    pub width: u32,
    pub height: u32,
}
