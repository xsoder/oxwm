use super::Block;
use anyhow::Result;
use std::fs;
use std::time::Duration;

pub struct Battery {
    format_charging: String,
    format_discharging: String,
    format_full: String,
    interval: Duration,
    color: u32,
    battery_path: String,
}

impl Battery {
    pub fn new(
        format_charging: &str,
        format_discharging: &str,
        format_full: &str,
        interval_secs: u64,
        color: u32,
    ) -> Self {
        Self {
            format_charging: format_charging.to_string(),
            format_discharging: format_discharging.to_string(),
            format_full: format_full.to_string(),
            interval: Duration::from_secs(interval_secs),
            color,
            battery_path: "/sys/class/power_supply/BAT0".to_string(),
        }
    }

    fn read_file(&self, filename: &str) -> Result<String> {
        let path = format!("{}/{}", self.battery_path, filename);
        Ok(fs::read_to_string(path)?.trim().to_string())
    }

    fn get_capacity(&self) -> Result<u32> {
        Ok(self.read_file("capacity")?.parse()?)
    }

    fn get_status(&self) -> Result<String> {
        self.read_file("status")
    }
}

impl Block for Battery {
    fn content(&mut self) -> Result<String> {
        let capacity = self.get_capacity()?;
        let status = self.get_status()?;

        let format = match status.as_str() {
            "Charging" => &self.format_charging,
            "Full" => &self.format_full,
            _ => &self.format_discharging,
        };

        Ok(format.replace("{}", &capacity.to_string()))
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    fn color(&self) -> u32 {
        self.color
    }
}
