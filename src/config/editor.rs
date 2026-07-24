use crate::managers::{ManagerMetadata, PACKAGE_MANAGERS};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;

pub struct RemovePackagesResult {
    pub removed: usize,
    pub not_found: Vec<String>,
}

enum ConfigEditOperation {
    Add,
    Remove,
}

pub fn add_packages_to_config(path: &Path, manager: &str, packages: &[String]) -> Result<usize> {
    let content = read_config(path)?;
    let mut doc = parse_config(&content)?;
    let (section, key) = manager_config_location(manager, ConfigEditOperation::Add)?;

    if doc.get(section).is_none() {
        doc[section] = toml_edit::table();
    }

    if doc[section].get(key).is_none() {
        doc[section][key] = toml_edit::array();
    }

    let array = doc[section][key]
        .as_array_mut()
        .context(format!("Expected array at [{}.{}]", section, key))?;

    let mut added = 0;
    for package in packages {
        if !array.iter().any(|value| value.as_str() == Some(package)) {
            array.push(package.as_str());
            added += 1;
        }
    }

    if added > 0 {
        write_config(path, &doc)?;
    }

    Ok(added)
}

pub fn remove_packages_from_config(
    path: &Path,
    manager: &str,
    packages: &[String],
) -> Result<RemovePackagesResult> {
    let content = read_config(path)?;
    let mut doc = parse_config(&content)?;
    let (section, key) = manager_config_location(manager, ConfigEditOperation::Remove)?;

    let requested: HashSet<&str> = packages.iter().map(String::as_str).collect();
    let mut removed_packages = Vec::new();

    if let Some(array) = doc
        .get_mut(section)
        .and_then(|section| section.get_mut(key))
        .and_then(|item| item.as_array_mut())
    {
        let mut index = 0;
        while index < array.len() {
            if let Some(value) = array.get(index).and_then(|value| value.as_str()) {
                if requested.contains(value) {
                    removed_packages.push(value.to_string());
                    array.remove(index);
                    continue;
                }
            }

            index += 1;
        }
    }

    if !removed_packages.is_empty() {
        write_config(path, &doc)?;
    }

    let removed_set: HashSet<&str> = removed_packages.iter().map(String::as_str).collect();
    let not_found = packages
        .iter()
        .filter(|package| !removed_set.contains(package.as_str()))
        .cloned()
        .collect();

    Ok(RemovePackagesResult {
        removed: removed_packages.len(),
        not_found,
    })
}

fn read_config(path: &Path) -> Result<String> {
    fs::read_to_string(path).context(format!("Failed to read config: {}", path.display()))
}

fn parse_config(content: &str) -> Result<DocumentMut> {
    content
        .parse::<DocumentMut>()
        .context("Failed to parse TOML")
}

fn write_config(path: &Path, doc: &DocumentMut) -> Result<()> {
    fs::write(path, doc.to_string()).context(format!("Failed to write config: {}", path.display()))
}

fn manager_config_location(
    manager: &str,
    operation: ConfigEditOperation,
) -> Result<(&'static str, &'static str)> {
    if let Some(meta) = ManagerMetadata::get_by_name(manager) {
        return match meta.name {
            "mas" => match operation {
                ConfigEditOperation::Add => anyhow::bail!(
                    "Adding mas apps via CLI not yet supported. Edit config manually."
                ),
                ConfigEditOperation::Remove => anyhow::bail!(
                    "Removing mas apps via CLI is not yet supported. Edit config manually."
                ),
            },
            "npm" => Ok(("npm", "global")),
            _ => Ok((meta.name, "packages")),
        };
    }

    match manager {
        "brew" => Ok(("brew", "formulae")),
        "cask" => Ok(("brew", "casks")),
        _ => {
            let available: Vec<_> = PACKAGE_MANAGERS.iter().map(|m| m.name).collect();
            anyhow::bail!(
                "Unknown manager: '{}'. Valid: brew, cask, {}",
                manager,
                available.join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn add_packages_creates_missing_section_and_skips_duplicates() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[brew]\nformulae = [\"ripgrep\"]\n").unwrap();

        let packages = vec!["ripgrep".to_string(), "fd".to_string()];
        let added = add_packages_to_config(&config_path, "brew", &packages).unwrap();

        assert_eq!(added, 1);
        let content = fs::read_to_string(config_path).unwrap();
        assert!(content.contains("formulae = [\"ripgrep\", \"fd\"]"));
    }

    #[test]
    fn remove_packages_reports_missing_packages() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[npm]\nglobal = [\"pnpm\", \"typescript\"]\n").unwrap();

        let packages = vec!["pnpm".to_string(), "eslint".to_string()];
        let result = remove_packages_from_config(&config_path, "npm", &packages).unwrap();

        assert_eq!(result.removed, 1);
        assert_eq!(result.not_found, vec!["eslint"]);
        let content = fs::read_to_string(config_path).unwrap();
        assert!(content.contains("typescript"));
        assert!(!content.contains("pnpm"));
    }
}
