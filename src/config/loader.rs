use super::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Find config file in order of priority:
/// 1. Explicit --config flag path
/// 2. ./macup.toml (current directory)
/// 3. ~/.config/macup/macup.toml
/// 4. ~/.macup.toml
pub fn find_config_file(explicit_path: Option<&Path>) -> Result<PathBuf> {
    // 1. Explicit path
    if let Some(path) = explicit_path {
        if path.exists() {
            return Ok(path.to_path_buf());
        }
        anyhow::bail!("Config file not found: {}", path.display());
    }

    // 2. Current directory
    let cwd_config = PathBuf::from("./macup.toml");
    if cwd_config.exists() {
        return Ok(cwd_config);
    }

    // 3. ~/.config/macup/macup.toml
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join("macup/macup.toml");
        if config_path.exists() {
            return Ok(config_path);
        }
    }

    // 4. ~/.macup.toml
    if let Some(home_dir) = dirs::home_dir() {
        let home_config = home_dir.join(".macup.toml");
        if home_config.exists() {
            return Ok(home_config);
        }
    }

    anyhow::bail!(
        "No config file found. Searched:\n\
         - ./macup.toml\n\
         - ~/.config/macup/macup.toml\n\
         - ~/.macup.toml"
    );
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
