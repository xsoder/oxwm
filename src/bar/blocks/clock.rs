use super::Block;
use anyhow::Result;
use chrono::Local;
use std::time::Duration;

pub struct Clock {
    format: &'static str,
    color: u32,
}

impl Clock {
    pub fn new(format: &'static str, color: u32) -> Self {
        Self { format, color }
    }
}

impl Block for Clock {
    fn content(&mut self) -> Result<String> {
        let now = Local::now();
        Ok(now.format(self.format).to_string())
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }

    fn color(&self) -> u32 {
        self.color
    }
}
