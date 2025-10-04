use super::Block;
use anyhow::Result;
use std::process::Command;
use std::time::Duration;

pub struct Uname {
    cached_version: String,
    prefix: &'static str,
    color: u32,
}

impl Uname {
    pub fn new(prefix: &'static str, color: u32) -> Self {
        Self {
            cached_version: String::new(),
            prefix,
            color,
        }
    }
}

impl Block for Uname {
    fn content(&mut self) -> Result<String> {
        if self.cached_version.is_empty() {
            let output = Command::new("uname").arg("-r").output()?;
            self.cached_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
        Ok(format!("{}{}", self.prefix, self.cached_version))
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(u64::MAX)
    }

    fn color(&self) -> u32 {
        self.color
    }
}
