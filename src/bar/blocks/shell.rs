use super::Block;
use anyhow::Result;
use std::process::Command;
use std::time::Duration;

pub struct ShellBlock {
    format: String,
    command: String,
    interval: Duration,
    color: u32,
}

impl ShellBlock {
    pub fn new(format: &str, command: &str, interval_secs: u64, color: u32) -> Self {
        Self {
            format: format.to_string(),
            command: command.to_string(),
            interval: Duration::from_secs(interval_secs),
            color,
        }
    }
}

impl Block for ShellBlock {
    fn content(&mut self) -> Result<String> {
        let output = Command::new("sh").arg("-c").arg(&self.command).output()?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(self.format.replace("{}", &result))
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    fn color(&self) -> u32 {
        self.color
    }
}
