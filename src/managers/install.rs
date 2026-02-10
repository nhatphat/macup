use crate::config::InstallScript;
use anyhow::Result;
use std::process::Command;

pub struct InstallManager;

impl InstallManager {
    pub fn new() -> Self {
        Self
    }

    pub fn apply_script(&self, script: &InstallScript) -> Result<()> {
        // Check if already installed
        if let Some(check_cmd) = &script.check {
            let check = Command::new("sh").arg("-c").arg(check_cmd).output()?;

            if check.status.success() {
                log::info!("✓ {} already installed", script.name);
                return Ok(());
            }
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
        if let Some(check_cmd) = &script.check {
            let verify = Command::new("sh").arg("-c").arg(check_cmd).output()?;

            if !verify.status.success() {
                anyhow::bail!("{} installed but verification failed", script.name);
            }
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
