mod bar;
mod blocks;
mod font;

pub use bar::Bar;
pub use blocks::{BlockCommand, BlockConfig};

// Bar position (for future use)
#[derive(Debug, Clone, Copy)]
pub enum BarPosition {
    Top,
    Bottom,
}
