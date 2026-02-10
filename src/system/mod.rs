use anyhow::Result;
use std::process::Command;

pub struct SystemManager;

impl SystemManager {
    pub fn new() -> Self {
        Self
    }

    pub fn apply_commands(&self, commands: &[String]) -> Result<()> {
        for cmd in commands {
            log::info!("→ Running: {}", cmd);

            let result = Command::new("sh").arg("-c").arg(cmd).status()?;

            if !result.success() {
                log::warn!("Command failed: {}", cmd);
            }
        }

        Ok(())
    }
}
