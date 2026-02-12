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

    /// Parse package name with optional binary mapping
    /// Format: "package:binary" or just "package"
    /// Examples:
    ///   - "typescript:tsc" -> install "typescript", check binary "tsc"
    ///   - "prettier" -> install "prettier", check binary "prettier"
    fn parse_package_name(input: &str) -> (&str, &str) {
        if let Some((pkg, bin)) = input.split_once(':') {
            (pkg.trim(), bin.trim())
        } else {
            (input.trim(), input.trim())
        }
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

    /// Install a global npm package
    /// Accepts "package:binary" format but only uses package name for installation
    pub fn install_global_package(&self, package_spec: &str) -> Result<()> {
        // Parse package:binary format - install using package name only
        let (pkg_name, _binary_name) = Self::parse_package_name(package_spec);

        log::info!("→ Installing {} (npm -g)...", pkg_name);

        let status = Command::new("npm")
            .args(["install", "-g", pkg_name])
            .status()
            .context(format!("Failed to install npm package: {}", pkg_name))?;

        if !status.success() {
            anyhow::bail!("npm install -g {} failed", pkg_name);
        }

        log::info!("✓ {} installed", pkg_name);
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

    fn is_package_installed(&self, package: &str) -> Result<bool> {
        // Parse package:binary format and check if binary exists
        let (_pkg_name, binary_name) = Self::parse_package_name(package);
        Ok(utils::command_exists(binary_name))
    }

    fn install_package(&self, package: &str) -> Result<()> {
        if self.is_package_installed(package)? {
            let (pkg_name, _) = Self::parse_package_name(package);
            log::info!("✓ {} already installed", pkg_name);
            return Ok(());
        }

        self.install_global_package(package)
    }

    fn install_packages(&self, packages: &[String]) -> Result<InstallResult> {
        if packages.is_empty() {
            return Ok(InstallResult::default());
        }

        // Check which packages are already installed by checking their binaries
        let to_install: Vec<_> = packages
            .iter()
            .filter(|pkg| {
                let (_pkg_name, binary_name) = Self::parse_package_name(pkg);
                !utils::command_exists(binary_name)
            })
            .cloned()
            .collect();

        let mut result = InstallResult::default();
        result.skipped = packages
            .iter()
            .filter(|pkg| {
                let (_pkg_name, binary_name) = Self::parse_package_name(pkg);
                utils::command_exists(binary_name)
            })
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
