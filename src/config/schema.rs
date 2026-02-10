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

    #[serde(default)]
    pub optional: Vec<String>,
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
