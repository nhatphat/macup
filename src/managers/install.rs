use crate::config::InstallScript;
use crate::utils::command::command_exists;
use anyhow::Result;
use std::process::Command;

pub struct InstallManager;

impl InstallManager {
    pub fn new() -> Self {
        Self
    }

    /// Check if script is already installed
    /// Priority: binary > check command
    pub fn is_installed(&self, script: &InstallScript) -> Result<bool> {
        // First, check binary if provided
        if let Some(binary) = &script.binary {
            if command_exists(binary) {
                return Ok(true);
            }
            // If binary specified but not found, try check command as fallback
        }

        // Fallback to check command
        if let Some(check_cmd) = &script.check {
            let check = Command::new("sh").arg("-c").arg(check_cmd).output()?;
            return Ok(check.status.success());
        }

        // If neither binary nor check provided, consider not installed
        Ok(false)
    }

    pub fn apply_script(&self, script: &InstallScript) -> Result<()> {
        // Check if already installed
        if self.is_installed(script)? {
            log::info!("✓ {} already installed", script.name);
            return Ok(());
        }

        // Run install command
        log::info!("→ Installing {}...", script.name);

        let result = Command::new("sh").arg("-c").arg(&script.command).status()?;

        if !result.success() {
            if script.required {
                anyhow::bail!("Failed to install {}", script.name);
            } else {
                log::warn!("Failed to install {} (optional)", script.name);
                return Ok(());
            }
        }

        // Verify installation
        if !self.is_installed(script)? {
            anyhow::bail!("{} installed but verification failed", script.name);
        }

        log::info!("✓ {} installed", script.name);
        Ok(())
    }

    pub fn apply_scripts(&self, scripts: &[InstallScript]) -> Result<()> {
        for script in scripts {
            if let Err(e) = self.apply_script(script) {
                if script.required {
                    return Err(e);
                } else {
                    log::warn!("Skipping optional script {}: {}", script.name, e);
                }
            }
        }
        Ok(())
    }
}
