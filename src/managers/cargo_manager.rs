use crate::managers::{InstallResult, Manager};
use crate::utils;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashSet;
use std::process::Command;

pub struct CargoManager {
    max_parallel: usize,
}

impl CargoManager {
    pub fn new(max_parallel: usize) -> Self {
        Self { max_parallel }
    }

    pub fn list_installed_packages(&self) -> Result<HashSet<String>> {
        let output = Command::new("cargo")
            .args(["install", "--list"])
            .output()
            .context("Failed to list cargo packages")?;

        let packages = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                // Lines with package names don't start with whitespace
                if !line.starts_with(char::is_whitespace) && line.contains(' ') {
                    line.split_whitespace().next().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    pub fn install_package_impl(&self, name: &str) -> Result<()> {
        log::info!("→ Installing {} (cargo)...", name);

        let status = Command::new("cargo")
            .args(["install", name])
            .status()
            .context(format!("Failed to install cargo package: {}", name))?;

        if !status.success() {
            anyhow::bail!("cargo install {} failed", name);
        }

        log::info!("✓ {} installed", name);
        Ok(())
    }
}

impl Manager for CargoManager {
    fn name(&self) -> &str {
        "cargo"
    }

    fn is_installed(&self) -> bool {
        utils::command_exists("cargo")
    }

    fn install_self(&self) -> Result<()> {
        anyhow::bail!("cargo not found. Install Rust first (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh)");
    }

    fn list_installed(&self) -> Result<HashSet<String>> {
        self.list_installed_packages()
    }

    fn install_package(&self, package: &str) -> Result<()> {
        if self.is_package_installed(package)? {
            log::info!("✓ {} already installed", package);
            return Ok(());
        }

        self.install_package_impl(package)
    }

    fn install_packages(&self, packages: &[String]) -> Result<InstallResult> {
        if packages.is_empty() {
            return Ok(InstallResult::default());
        }

        let installed = self.list_installed_packages()?;
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
            log::info!(
                "✓ {} cargo packages already installed",
                result.skipped.len()
            );
        }

        if to_install.is_empty() {
            return Ok(result);
        }

        log::info!("Installing {} cargo packages...", to_install.len());

        let results: Vec<_> = rayon::ThreadPoolBuilder::new()
            .num_threads(self.max_parallel)
            .build()?
            .install(|| {
                to_install
                    .par_iter()
                    .map(|pkg| (pkg.clone(), self.install_package_impl(pkg)))
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
