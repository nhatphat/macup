pub mod brew;
pub mod cargo_manager;
pub mod install;
pub mod mas;
pub mod npm;

use anyhow::Result;
use std::collections::HashSet;

/// Result of installing packages
#[derive(Debug, Default)]
pub struct InstallResult {
    pub success: Vec<String>,
    pub failed: Vec<(String, String)>, // (package, error)
    pub skipped: Vec<String>,
}

/// Trait for package managers
pub trait Manager {
    /// Manager name (brew, mas, npm, cargo)
    fn name(&self) -> &str;

    /// Check if manager is installed
    fn is_installed(&self) -> bool;

    /// Install the manager itself
    #[allow(dead_code)]
    fn install_self(&self) -> Result<()>;

    /// Get list of currently installed packages
    fn list_installed(&self) -> Result<HashSet<String>>;

    /// Check if a specific package is installed
    fn is_package_installed(&self, package: &str) -> Result<bool> {
        Ok(self.list_installed()?.contains(package))
    }

    /// Install a single package (with idempotency check)
    fn install_package(&self, package: &str) -> Result<()>;

    /// Install multiple packages (batch check + parallel install)
    fn install_packages(&self, packages: &[String]) -> Result<InstallResult>;
}
