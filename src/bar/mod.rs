mod bar;
mod font;
// mod widgets;  // TODO: implement later

pub use bar::Bar;

// TODO: this should live in config.rs
pub const BAR_HEIGHT: u16 = 25;

// Bar position (for future use)
#[derive(Debug, Clone, Copy)]
pub enum BarPosition {
    Top,
    Bottom,
}
