use super::Block;
use crate::errors::BlockError;
use std::fs;
use std::time::Duration;

pub struct Ram {
    format: String,
    interval: Duration,
    color: u32,
}

impl Ram {
    pub fn new(format: &str, interval_secs: u64, color: u32) -> Self {
        Self {
            format: format.to_string(),
            interval: Duration::from_secs(interval_secs),
            color,
        }
    }

    fn get_memory_info(&self) -> Result<(u64, u64, f32), BlockError> {
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        let mut total: u64 = 0;
        let mut available: u64 = 0;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0)
            } else if line.starts_with("MemAvailable:") {
                available = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0)
            }
        }

        let used = total.saturating_sub(available);
        let percentage = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        Ok((used, total, percentage))
    }
}

impl Block for Ram {
    fn content(&mut self) -> Result<String, BlockError> {
        let (used, total, percentage) = self.get_memory_info()?;

        let used_gb = used as f32 / 1024.0 / 1024.0;
        let total_gb = total as f32 / 1024.0 / 1024.0;

        let result = self
            .format
            .replace("{used}", &format!("{:.1}", used_gb))
            .replace("{total}", &format!("{:.1}", total_gb))
            .replace("{percent}", &format!("{:.1}", percentage))
            .replace("{}", &format!("{:.1}", used_gb));

        Ok(result)
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    fn color(&self) -> u32 {
        self.color
    }
}
