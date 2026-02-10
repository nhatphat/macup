use crate::managers::{InstallResult, Manager};
use crate::utils;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashSet;
use std::process::Command;

pub struct NpmManager {
    max_parallel: usize,
}

impl NpmManager {
    pub fn new(max_parallel: usize) -> Self {
        Self { max_parallel }
    }

    pub fn list_global_packages(&self) -> Result<HashSet<String>> {
        let output = Command::new("npm")
            .args(["list", "-g", "--depth=0", "--parseable"])
            .output()
            .context("Failed to list npm global packages")?;

        let packages = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                // Extract package name from path
                line.split('/').last().map(|s| s.to_string())
            })
            .collect();

        Ok(packages)
    }

    pub fn install_global_package(&self, name: &str) -> Result<()> {
        log::info!("→ Installing {} (npm -g)...", name);

        let status = Command::new("npm")
            .args(["install", "-g", name])
            .status()
            .context(format!("Failed to install npm package: {}", name))?;

        if !status.success() {
            anyhow::bail!("npm install -g {} failed", name);
        }

        log::info!("✓ {} installed", name);
        Ok(())
    }
}

impl Manager for NpmManager {
    fn name(&self) -> &str {
        "npm"
    }

    fn is_installed(&self) -> bool {
        utils::command_exists("npm")
    }

    fn install_self(&self) -> Result<()> {
        anyhow::bail!("npm not found. Install Node.js first (via brew install node)");
    }

    fn list_installed(&self) -> Result<HashSet<String>> {
        self.list_global_packages()
    }

    fn install_package(&self, package: &str) -> Result<()> {
        if self.is_package_installed(package)? {
            log::info!("✓ {} already installed", package);
            return Ok(());
        }

        self.install_global_package(package)
    }

    fn install_packages(&self, packages: &[String]) -> Result<InstallResult> {
        if packages.is_empty() {
            return Ok(InstallResult::default());
        }

        let installed = self.list_global_packages()?;
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
            log::info!("✓ {} npm packages already installed", result.skipped.len());
        }

        if to_install.is_empty() {
            return Ok(result);
        }

        log::info!("Installing {} npm packages...", to_install.len());

        let results: Vec<_> = rayon::ThreadPoolBuilder::new()
            .num_threads(self.max_parallel)
            .build()?
            .install(|| {
                to_install
                    .par_iter()
                    .map(|pkg| (pkg.clone(), self.install_global_package(pkg)))
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
