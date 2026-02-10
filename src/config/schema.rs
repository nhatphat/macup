use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,

    #[serde(default)]
    pub managers: Managers,

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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Managers {
    #[serde(default)]
    pub required: Vec<String>,
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
    /// Returns managers that MUST be installed based on declared packages
    pub fn detect_required_managers(&self) -> Vec<String> {
        let mut managers = Vec::new();

        // Check brew section - if has any packages, brew is required
        if let Some(brew) = &self.brew {
            if !brew.taps.is_empty() || !brew.formulae.is_empty() || !brew.casks.is_empty() {
                managers.push("brew".to_string());
            }
        }

        // Note: mas, npm, and cargo auto-install their runtimes inline in their sections
        // Only brew needs to be in the managers list (foundation)

        managers
    }

    /// Get final list of required managers (explicit + auto-detected)
    pub fn get_required_managers(&self) -> Vec<String> {
        let mut all = self.managers.required.clone();

        // Add auto-detected managers if not already in list
        for manager in self.detect_required_managers() {
            if !all.contains(&manager) {
                all.push(manager);
            }
        }

        all
    }
}
