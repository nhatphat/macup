use crate::managers::{InstallResult, Manager};
use crate::utils;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashSet;
use std::process::Command;

pub struct MasManager {
    max_parallel: usize,
}

impl MasManager {
    pub fn new(max_parallel: usize) -> Self {
        Self { max_parallel }
    }

    pub fn list_apps(&self) -> Result<HashSet<String>> {
        let output = Command::new("mas")
            .arg("list")
            .output()
            .context("Failed to run mas list")?;

        if !output.status.success() {
            anyhow::bail!("mas list failed");
        }

        let apps = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                // Format: "ID Name"
                line.split_whitespace().next().map(|s| s.to_string())
            })
            .collect();

        Ok(apps)
    }

    pub fn install_app(&self, id: &str) -> Result<()> {
        log::info!("→ Installing app {}...", id);

        let status = Command::new("mas")
            .args(["install", id])
            .status()
            .context(format!("Failed to install app: {}", id))?;

        if !status.success() {
            anyhow::bail!("mas install {} failed", id);
        }

        log::info!("✓ App {} installed", id);
        Ok(())
    }
}

impl Manager for MasManager {
    fn name(&self) -> &str {
        "mas"
    }

    fn is_installed(&self) -> bool {
        utils::command_exists("mas")
    }

    fn install_self(&self) -> Result<()> {
        log::info!("Installing mas-cli via Homebrew...");
        Command::new("brew")
            .env("HOMEBREW_NO_AUTO_UPDATE", "1")
            .args(["install", "mas"])
            .status()?;
        Ok(())
    }

    fn list_installed(&self) -> Result<HashSet<String>> {
        self.list_apps()
    }

    fn install_package(&self, package: &str) -> Result<()> {
        if self.is_package_installed(package)? {
            log::info!("✓ App {} already installed", package);
            return Ok(());
        }

        self.install_app(package)
    }

    fn install_packages(&self, packages: &[String]) -> Result<InstallResult> {
        if packages.is_empty() {
            return Ok(InstallResult::default());
        }

        let installed = self.list_apps()?;
        let to_install: Vec<_> = packages
            .iter()
            .filter(|pkg| !installed.contains(pkg.as_str()))
            .cloned()
            .collect();

        let mut result = InstallResult::default();
        result.skipped = packages
            .iter()
            .filter(|pkg| installed.contains(pkg.as_str()))
            .cloned()
            .collect();

        if !result.skipped.is_empty() {
            log::info!("✓ {} apps already installed", result.skipped.len());
        }

        if to_install.is_empty() {
            return Ok(result);
        }

        log::info!("Installing {} apps...", to_install.len());

        let results: Vec<_> = rayon::ThreadPoolBuilder::new()
            .num_threads(self.max_parallel)
            .build()?
            .install(|| {
                to_install
                    .par_iter()
                    .map(|pkg| (pkg.clone(), self.install_app(pkg)))
                    .collect()
            });

        for (pkg, res) in results {
            match res {
                Ok(_) => result.success.push(pkg),
                Err(e) => result.failed.push((pkg, e.to_string())),
            }
        }

        Ok(result)
    }
}
