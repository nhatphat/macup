use crate::config::find_config_file;
use crate::managers::{ManagerMetadata, PACKAGE_MANAGERS};
use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;

pub fn run(config_path: Option<&Path>, manager: &str, packages: Vec<String>) -> Result<()> {
    if packages.is_empty() {
        anyhow::bail!("No packages specified");
    }

    println!(
        "{}",
        format!(
            "Removing {} package(s) from [{}] config...",
            packages.len(),
            manager
        )
        .bright_cyan()
    );
    println!();

    let config_file = find_config_file(config_path)?;
    let result = update_config_file(&config_file, manager, &packages)?;

    if result.removed > 0 {
        println!(
            "{}",
            format!("✓ Removed {} package(s) from config", result.removed).green()
        );
    }

    if !result.not_found.is_empty() {
        println!();
        println!(
            "{}",
            format!("{} package(s) were not in config:", result.not_found.len()).yellow()
        );
        for package in result.not_found {
            println!("  - {}", package);
        }
    }

    Ok(())
}

struct RemoveResult {
    removed: usize,
    not_found: Vec<String>,
}

fn update_config_file(path: &Path, manager: &str, packages: &[String]) -> Result<RemoveResult> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read config: {}", path.display()))?;

    let mut doc = content
        .parse::<DocumentMut>()
        .context("Failed to parse TOML")?;

    let (section, key) = manager_config_location(manager)?;

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
        fs::write(path, doc.to_string())
            .context(format!("Failed to write config: {}", path.display()))?;
    }

    let removed_set: HashSet<&str> = removed_packages.iter().map(String::as_str).collect();
    let not_found = packages
        .iter()
        .filter(|pkg| !removed_set.contains(pkg.as_str()))
        .cloned()
        .collect();

    Ok(RemoveResult {
        removed: removed_packages.len(),
        not_found,
    })
}

fn manager_config_location(manager: &str) -> Result<(&'static str, &'static str)> {
    if let Some(meta) = ManagerMetadata::get_by_name(manager) {
        return match meta.name {
            "mas" => anyhow::bail!(
                "Removing mas apps via CLI is not yet supported. Edit config manually."
            ),
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
