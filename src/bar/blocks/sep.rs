use super::Block;
use anyhow::Result;
use std::time::Duration;

pub struct Sep {
    text: &'static str,
    color: u32,
}

impl Sep {
    pub fn new(text: &'static str, color: u32) -> Self {
        Self { text, color }
    }
}

impl Block for Sep {
    fn content(&mut self) -> Result<String> {
        Ok(self.text.to_string())
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(u64::MAX)
    }

    fn color(&self) -> u32 {
        self.color
    }
}
