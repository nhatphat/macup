use super::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Find config file in order of priority:
/// 1. Explicit --config flag path
/// 2. ~/.config/macup/config.toml
pub fn find_config_file(explicit_path: Option<&Path>) -> Result<PathBuf> {
    // 1. Explicit path
    if let Some(path) = explicit_path {
        if path.exists() {
            return Ok(path.to_path_buf());
        }
        anyhow::bail!("Config file not found: {}", path.display());
    }

    let config_path = default_config_path()?;
    if config_path.exists() {
        return Ok(config_path);
    }

    anyhow::bail!("No config file found at {}", config_path.display());
}

/// Default XDG config path used by macup.
pub fn default_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Failed to determine user config directory")?;
    Ok(config_dir.join("macup").join("config.toml"))
}

/// Load and parse config file
pub fn load_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML config: {}", path.display()))?;

    Ok(config)
}

/// Load config with automatic discovery
pub fn load_config_auto(explicit_path: Option<&Path>) -> Result<(PathBuf, Config)> {
    let path = find_config_file(explicit_path)?;
    let config = load_config(&path)?;
    Ok((path, config))
}
