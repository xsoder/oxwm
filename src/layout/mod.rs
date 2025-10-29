pub mod normie;
pub mod tiling;

use x11rb::protocol::xproto::Window;

pub type LayoutBox = Box<dyn Layout>;

pub struct GapConfig {
    pub inner_horizontal: u32,
    pub inner_vertical: u32,
    pub outer_horizontal: u32,
    pub outer_vertical: u32,
}

pub enum LayoutType {
    Tiling,
    Normie,
}

impl LayoutType {
    pub fn new(&self) -> LayoutBox {
        match self {
            Self::Tiling => Box::new(tiling::TilingLayout),
            Self::Normie => Box::new(normie::NormieLayout),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Tiling => Self::Normie,
            Self::Normie => Self::Tiling,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tiling => "tiling",
            Self::Normie => "normie",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "tiling" => Ok(Self::Tiling),
            "normie" | "floating" => Ok(Self::Normie),
            _ => Err(format!("Invalid Layout Type: {}", s)),
        }
    }
}

pub fn layout_from_str(s: &str) -> Result<LayoutBox, String> {
    let layout_type = LayoutType::from_str(s)?;
    Ok(layout_type.new())
}

pub fn next_layout(current_name: &str) -> &'static str {
    LayoutType::from_str(current_name)
        .ok()
        .map(|layout_type| layout_type.next())
        .unwrap_or(LayoutType::Tiling)
        .as_str()
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
