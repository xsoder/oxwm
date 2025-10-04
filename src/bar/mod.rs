mod bar;
mod blocks;
mod font;
// mod widgets;  // TODO: implement later

pub use bar::Bar;

// Bar position (for future use)
#[derive(Debug, Clone, Copy)]
pub enum BarPosition {
    Top,
    Bottom,
}
