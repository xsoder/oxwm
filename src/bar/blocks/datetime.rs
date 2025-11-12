use super::Block;
use crate::errors::BlockError;
use chrono::Local;
use std::time::Duration;

pub struct DateTime {
    format_template: String,
    time_format: String,
    interval: Duration,
    color: u32,
}

impl DateTime {
    pub fn new(format_template: &str, time_format: &str, interval_secs: u64, color: u32) -> Self {
        Self {
            format_template: format_template.to_string(),
            time_format: time_format.to_string(),
            interval: Duration::from_secs(interval_secs),
            color,
        }
    }
}

impl Block for DateTime {
    fn content(&mut self) -> Result<String, BlockError> {
        let now = Local::now();
        let time_str = now.format(&self.time_format).to_string();
        Ok(self.format_template.replace("{}", &time_str))
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    fn color(&self) -> u32 {
        self.color
    }
}
