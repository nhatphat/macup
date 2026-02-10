use crate::managers::{InstallResult, Manager};
use crate::utils;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashSet;
use std::process::Command;

pub struct BrewManager {
    max_parallel: usize,
}

impl BrewManager {
    pub fn new(max_parallel: usize) -> Self {
        Self { max_parallel }
    }

    /// Create brew command with HOMEBREW_NO_AUTO_UPDATE=1
    fn brew_command(&self) -> Command {
        let mut cmd = Command::new("brew");
        cmd.env("HOMEBREW_NO_AUTO_UPDATE", "1");
        cmd
    }

    /// List installed formulae
    pub fn list_formulae(&self) -> Result<HashSet<String>> {
        let output = self
            .brew_command()
            .args(["list", "--formula"])
            .output()
            .context("Failed to list brew formulae")?;

        if !output.status.success() {
            anyhow::bail!("brew list --formula failed");
        }

        let installed = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(installed)
    }

    /// List installed casks
    pub fn list_casks(&self) -> Result<HashSet<String>> {
        let output = self
            .brew_command()
            .args(["list", "--cask"])
            .output()
            .context("Failed to list brew casks")?;

        if !output.status.success() {
            anyhow::bail!("brew list --cask failed");
        }

        let installed = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(installed)
    }

    /// List installed taps
    pub fn list_taps(&self) -> Result<HashSet<String>> {
        let output = self
            .brew_command()
            .arg("tap")
            .output()
            .context("Failed to list brew taps")?;

        if !output.status.success() {
            anyhow::bail!("brew tap failed");
        }

        let taps = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(taps)
    }

    /// Install a formula
    pub fn install_formula(&self, name: &str) -> Result<()> {
        log::info!("→ Installing {} (formula)...", name);

        let status = self
            .brew_command()
            .args(["install", name])
            .status()
            .context(format!("Failed to install formula: {}", name))?;

        if !status.success() {
            anyhow::bail!("brew install {} failed", name);
        }

        log::info!("✓ {} installed", name);
        Ok(())
    }

    /// Install a cask
    pub fn install_cask(&self, name: &str) -> Result<()> {
        log::info!("→ Installing {} (cask)...", name);

        let status = self
            .brew_command()
            .args(["install", "--cask", name])
            .status()
            .context(format!("Failed to install cask: {}", name))?;

        if !status.success() {
            anyhow::bail!("brew install --cask {} failed", name);
        }

        log::info!("✓ {} installed", name);
        Ok(())
    }

    /// Add a tap
    pub fn add_tap(&self, name: &str) -> Result<()> {
        log::info!("→ Adding tap {}...", name);

        let status = self
            .brew_command()
            .args(["tap", name])
            .status()
            .context(format!("Failed to add tap: {}", name))?;

        if !status.success() {
            anyhow::bail!("brew tap {} failed", name);
        }

        log::info!("✓ Tap {} added", name);
        Ok(())
    }

    /// Install formulae with idempotency
    pub fn install_formulae(&self, formulae: &[String]) -> Result<InstallResult> {
        if formulae.is_empty() {
            return Ok(InstallResult::default());
        }

        log::info!("Checking {} formulae...", formulae.len());

        // Batch check installed
        let installed = self.list_formulae()?;

        // Filter to only packages that need installation
        let to_install: Vec<_> = formulae
            .iter()
            .filter(|pkg| !installed.contains(pkg.as_str()))
            .cloned()
            .collect();

        let mut result = InstallResult::default();
        result.skipped = formulae
            .iter()
            .filter(|pkg| installed.contains(pkg.as_str()))
            .cloned()
            .collect();

        if !result.skipped.is_empty() {
            log::info!("✓ {} formulae already installed", result.skipped.len());
        }

        if to_install.is_empty() {
            return Ok(result);
        }

        log::info!("Installing {} formulae...", to_install.len());

        // Parallel install
        let results: Vec<_> = rayon::ThreadPoolBuilder::new()
            .num_threads(self.max_parallel)
            .build()?
            .install(|| {
                to_install
                    .par_iter()
                    .map(|pkg| (pkg.clone(), self.install_formula(pkg)))
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

    /// Install casks with idempotency
    pub fn install_casks(&self, casks: &[String]) -> Result<InstallResult> {
        if casks.is_empty() {
            return Ok(InstallResult::default());
        }

        log::info!("Checking {} casks...", casks.len());

        let installed = self.list_casks()?;

        let to_install: Vec<_> = casks
            .iter()
            .filter(|pkg| !installed.contains(pkg.as_str()))
            .cloned()
            .collect();

        let mut result = InstallResult::default();
        result.skipped = casks
            .iter()
            .filter(|pkg| installed.contains(pkg.as_str()))
            .cloned()
            .collect();

        if !result.skipped.is_empty() {
            log::info!("✓ {} casks already installed", result.skipped.len());
        }

        if to_install.is_empty() {
            return Ok(result);
        }

        log::info!("Installing {} casks...", to_install.len());

        let results: Vec<_> = rayon::ThreadPoolBuilder::new()
            .num_threads(self.max_parallel)
            .build()?
            .install(|| {
                to_install
                    .par_iter()
                    .map(|pkg| (pkg.clone(), self.install_cask(pkg)))
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

    /// Add taps
    pub fn add_taps(&self, taps: &[String]) -> Result<InstallResult> {
        if taps.is_empty() {
            return Ok(InstallResult::default());
        }

        log::info!("Checking {} taps...", taps.len());

        let installed = self.list_taps()?;

        let to_add: Vec<_> = taps
            .iter()
            .filter(|tap| !installed.contains(tap.as_str()))
            .cloned()
            .collect();

        let mut result = InstallResult::default();
        result.skipped = taps
            .iter()
            .filter(|tap| installed.contains(tap.as_str()))
            .cloned()
            .collect();

        if !result.skipped.is_empty() {
            log::info!("✓ {} taps already added", result.skipped.len());
        }

        if to_add.is_empty() {
            return Ok(result);
        }

        // Taps are added sequentially (safer)
        for tap in to_add {
            match self.add_tap(&tap) {
                Ok(_) => result.success.push(tap),
                Err(e) => result.failed.push((tap, e.to_string())),
            }
        }

        Ok(result)
    }
}

impl Manager for BrewManager {
    fn name(&self) -> &str {
        "brew"
    }

    fn is_installed(&self) -> bool {
        utils::command_exists("brew")
    }

    fn install_self(&self) -> Result<()> {
        log::info!("Installing Homebrew...");
        anyhow::bail!("Homebrew not installed. Please run:\n/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"");
    }

    fn list_installed(&self) -> Result<HashSet<String>> {
        self.list_formulae()
    }

    fn install_package(&self, package: &str) -> Result<()> {
        if self.is_package_installed(package)? {
            log::info!("✓ {} already installed", package);
            return Ok(());
        }

        self.install_formula(package)
    }

    fn install_packages(&self, packages: &[String]) -> Result<InstallResult> {
        self.install_formulae(packages)
    }
}
