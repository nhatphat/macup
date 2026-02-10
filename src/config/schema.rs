use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,

    #[serde(default)]
    pub brew: Option<BrewConfig>,

    #[serde(default)]
    pub mas: Option<MasConfig>,

    #[serde(default)]
    pub npm: Option<NpmConfig>,

    #[serde(default)]
    pub cargo: Option<CargoConfig>,

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NpmConfig {
    #[serde(default)]
    pub depends_on: Vec<String>,

    #[serde(default)]
    pub global: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CargoConfig {
    #[serde(default)]
    pub depends_on: Vec<String>,

    #[serde(default)]
    pub packages: Vec<String>,
}

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
    /// Auto-detect required managers from config sections
    /// Returns managers that MUST be installed based on declared packages or dependencies
    pub fn detect_required_managers(&self) -> Vec<String> {
        let mut managers = Vec::new();

        // Check brew section - if has any packages, brew is required
        if let Some(brew) = &self.brew {
            if !brew.taps.is_empty() || !brew.formulae.is_empty() || !brew.casks.is_empty() {
                managers.push("brew".to_string());
            }
        }

        // Check if any section depends on brew
        let mas_needs_brew = self
            .mas
            .as_ref()
            .map_or(false, |m| m.depends_on.contains(&"brew".to_string()));
        let npm_needs_brew = self
            .npm
            .as_ref()
            .map_or(false, |n| n.depends_on.contains(&"brew".to_string()));
        let cargo_needs_brew = self
            .cargo
            .as_ref()
            .map_or(false, |c| c.depends_on.contains(&"brew".to_string()));
        let install_needs_brew = self
            .install
            .as_ref()
            .map_or(false, |i| i.depends_on.contains(&"brew".to_string()));
        let system_needs_brew = self
            .system
            .as_ref()
            .map_or(false, |s| s.depends_on.contains(&"brew".to_string()));

        let needs_brew = mas_needs_brew
            || npm_needs_brew
            || cargo_needs_brew
            || install_needs_brew
            || system_needs_brew;

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
