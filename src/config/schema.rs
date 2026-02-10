use serde::{Deserialize, Serialize};

/// Trait for package manager config sections (mas, npm, cargo, etc.)
/// Allows generic iteration over different manager types
pub trait PackageManagerSection {
    /// Get the dependencies this section requires
    fn get_depends_on(&self) -> &Vec<String>;

    /// Check if this section has any packages to install
    #[allow(dead_code)]
    fn has_packages(&self) -> bool;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,

    #[serde(default)]
    pub brew: Option<BrewConfig>,

    // CODEGEN_START[mas]: config_field
    #[serde(default)]
    pub mas: Option<MasConfig>,
    // CODEGEN_END[mas]: config_field

    // CODEGEN_START[npm]: config_field
    #[serde(default)]
    pub npm: Option<NpmConfig>,
    // CODEGEN_END[npm]: config_field

    // CODEGEN_START[cargo]: config_field
    #[serde(default)]
    pub cargo: Option<CargoConfig>,
    // CODEGEN_END[cargo]: config_field









    // CODEGEN_MARKER: insert_config_field_here
    #[serde(default)]
    pub install: Option<InstallConfig>,

    #[serde(default)]
    pub system: Option<SystemConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default)]
    pub fail_fast: bool,

    #[serde(default = "default_max_parallel")]
    pub max_parallel: usize,
}

fn default_max_parallel() -> usize {
    4
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fail_fast: false,
            max_parallel: default_max_parallel(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrewConfig {
    #[serde(default)]
    pub depends_on: Vec<String>,

    #[serde(default)]
    pub taps: Vec<String>,

    #[serde(default)]
    pub formulae: Vec<String>,

    #[serde(default)]
    pub casks: Vec<String>,
}

// CODEGEN_START[mas]: config_struct
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MasConfig {
    #[serde(default)]
    pub depends_on: Vec<String>,

    #[serde(default)]
    pub apps: Vec<MasApp>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MasApp {
    pub name: String,
    pub id: u64,
}

impl PackageManagerSection for MasConfig {
    fn get_depends_on(&self) -> &Vec<String> {
        &self.depends_on
    }

    fn has_packages(&self) -> bool {
        !self.apps.is_empty()
    }
}
// CODEGEN_END[mas]: config_struct

// CODEGEN_START[npm]: config_struct
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NpmConfig {
    #[serde(default)]
    pub depends_on: Vec<String>,

    #[serde(default)]
    pub global: Vec<String>,
}

impl PackageManagerSection for NpmConfig {
    fn get_depends_on(&self) -> &Vec<String> {
        &self.depends_on
    }

    fn has_packages(&self) -> bool {
        !self.global.is_empty()
    }
}
// CODEGEN_END[npm]: config_struct

// CODEGEN_START[cargo]: config_struct
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CargoConfig {
    #[serde(default)]
    pub depends_on: Vec<String>,

    #[serde(default)]
    pub packages: Vec<String>,
}

impl PackageManagerSection for CargoConfig {
    fn get_depends_on(&self) -> &Vec<String> {
        &self.depends_on
    }

    fn has_packages(&self) -> bool {
        !self.packages.is_empty()
    }
}
// CODEGEN_END[cargo]: config_struct









// CODEGEN_MARKER: insert_config_struct_here

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstallConfig {
    #[serde(default)]
    pub depends_on: Vec<String>,

    #[serde(default)]
    pub scripts: Vec<InstallScript>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstallScript {
    pub name: String,

    #[serde(default)]
    pub check: Option<String>,

    pub command: String,

    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemConfig {
    #[serde(default)]
    pub depends_on: Vec<String>,

    #[serde(default)]
    pub commands: Vec<String>,
}

impl Config {
    /// Get package manager config by name (generic accessor)
    pub fn get_manager_config(&self, name: &str) -> Option<&dyn PackageManagerSection> {
        match name {
            // CODEGEN_START[mas]: match_arm
            "mas" => self.mas.as_ref().map(|c| c as &dyn PackageManagerSection),
            // CODEGEN_END[mas]: match_arm
            // CODEGEN_START[npm]: match_arm
            "npm" => self.npm.as_ref().map(|c| c as &dyn PackageManagerSection),
            // CODEGEN_END[npm]: match_arm
            // CODEGEN_START[cargo]: match_arm
            "cargo" => self.cargo.as_ref().map(|c| c as &dyn PackageManagerSection),
            // CODEGEN_END[cargo]: match_arm
            // CODEGEN_MARKER: insert_manager_match_arm_here
            _ => None,
        }
    }

    /// Auto-detect required managers from config sections
    /// Returns managers that MUST be installed based on declared packages or dependencies
    pub fn detect_required_managers(&self) -> Vec<String> {
        use crate::managers::PACKAGE_MANAGERS;
        let mut managers = Vec::new();

        // Check brew section - if has any packages, brew is required
        if let Some(brew) = &self.brew {
            if !brew.taps.is_empty() || !brew.formulae.is_empty() || !brew.casks.is_empty() {
                managers.push("brew".to_string());
            }
        }

        // Check if any package manager section depends on brew
        let mut needs_brew = false;
        for meta in PACKAGE_MANAGERS {
            if let Some(config) = self.get_manager_config(meta.name) {
                if config.get_depends_on().contains(&"brew".to_string()) {
                    needs_brew = true;
                    break;
                }
            }
        }

        // Also check install and system sections
        needs_brew = needs_brew
            || self
                .install
                .as_ref()
                .map_or(false, |i| i.depends_on.contains(&"brew".to_string()))
            || self
                .system
                .as_ref()
                .map_or(false, |s| s.depends_on.contains(&"brew".to_string()));

        if needs_brew && !managers.contains(&"brew".to_string()) {
            managers.push("brew".to_string());
        }

        // Note: mas, npm, and cargo auto-install their runtimes inline in their sections
        // But brew is the foundation - must be available if anything depends on it

        managers
    }

    /// Get list of required managers (auto-detected only)
    pub fn get_required_managers(&self) -> Vec<String> {
        self.detect_required_managers()
    }
}
