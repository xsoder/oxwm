use anyhow::Result;
use std::time::Duration;

pub mod clock;
pub mod sep;
pub mod uname;

pub use clock::Clock;
pub use sep::Sep;
pub use uname::Uname;

pub trait Block {
    fn content(&mut self) -> Result<String>;
    fn interval(&self) -> Duration;
    fn color(&self) -> u32;

    fn on_click(&mut self) -> Result<()> {
        Ok(())
    }
}
