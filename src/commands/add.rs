use crate::config::{find_config_file, load_config};
use crate::managers::{
    brew::BrewManager, cargo_manager::CargoManager, mas::MasManager, npm::NpmManager, Manager,
};
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;

pub fn run(
    config_path: Option<&Path>,
    manager: &str,
    packages: Vec<String>,
    no_install: bool,
) -> Result<()> {
    if packages.is_empty() {
        anyhow::bail!("No packages specified");
    }

    println!(
        "{}",
        format!("Adding {} package(s) to [{}]...", packages.len(), manager).bright_cyan()
    );
    println!();

    // Find config file
    let config_file = find_config_file(config_path)?;

    // Load config to check dependencies
    let config = load_config(&config_file)?;

    // Determine max_parallel
    let max_parallel = config.settings.max_parallel;

    // Get manager instance
    let mgr: Box<dyn Manager> = match manager {
        "brew" => Box::new(BrewManager::new(max_parallel)),
        "cask" => Box::new(BrewManager::new(max_parallel)),
        "mas" => Box::new(MasManager::new(max_parallel)),
        "npm" => Box::new(NpmManager::new(max_parallel)),
        "cargo" => Box::new(CargoManager::new(max_parallel)),
        _ => anyhow::bail!(
            "Unknown manager: {}. Valid: brew, cask, mas, npm, cargo",
            manager
        ),
    };

    // Check if manager is installed
    if !mgr.is_installed() {
        anyhow::bail!("{} is not installed. Run 'macup apply' first.", mgr.name());
    }

    // Install packages first, collect successful ones
    let mut to_add = Vec::new();
    let mut errors = Vec::new();

    for package in &packages {
        print!("→ Checking {}... ", package);

        if !no_install {
            // Check if already installed
            if mgr.is_package_installed(package).unwrap_or(false) {
                println!("{}", "already installed".green());
                to_add.push(package.clone());
                continue;
            }

            // Install
            print!("installing... ");
            match mgr.install_package(package) {
                Ok(_) => {
                    println!("{}", "✓".green());
                    to_add.push(package.clone());
                }
                Err(e) => {
                    println!("{}", format!("✗ {}", e).red());
                    errors.push((package.clone(), e));
                }
            }
        } else {
            // --no-install: just add to config
            println!("skipping install");
            to_add.push(package.clone());
        }
    }

    // Update config
    if !to_add.is_empty() {
        println!();
        println!("Updating config...");
        update_config_file(&config_file, manager, &to_add)?;
        println!(
            "{}",
            format!("✓ Added {} package(s) to config", to_add.len()).green()
        );
    }

    // Report errors
    if !errors.is_empty() {
        println!();
        println!(
            "{}",
            format!("⚠ {} package(s) failed to install:", errors.len()).yellow()
        );
        for (pkg, err) in errors {
            println!("  - {}: {}", pkg, err);
        }
    }

    Ok(())
}

fn update_config_file(path: &Path, manager: &str, packages: &[String]) -> Result<()> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read config: {}", path.display()))?;

    let mut doc = content
        .parse::<DocumentMut>()
        .context("Failed to parse TOML")?;

    // Determine section and key
    let (section, key) = match manager {
        "brew" => ("brew", "formulae"),
        "cask" => ("brew", "casks"),
        "npm" => ("npm", "global"),
        "cargo" => ("cargo", "packages"),
        "mas" => {
            // Special case: mas needs ID format
            anyhow::bail!("Adding mas apps via CLI not yet supported. Edit config manually.");
        }
        _ => anyhow::bail!("Unknown manager: {}", manager),
    };

    // Get or create section
    if doc.get(section).is_none() {
        doc[section] = toml_edit::table();
    }

    // Get or create array
    if doc[section].get(key).is_none() {
        doc[section][key] = toml_edit::array();
    }

    let array = doc[section][key]
        .as_array_mut()
        .context(format!("Expected array at [{}.{}]", section, key))?;

    // Add packages
    let mut added = 0;
    for pkg in packages {
        // Check if already in config
        if !array.iter().any(|v| v.as_str() == Some(pkg)) {
            array.push(pkg.as_str());
            added += 1;
        }
    }

    if added > 0 {
        fs::write(path, doc.to_string())
            .context(format!("Failed to write config: {}", path.display()))?;
    }

    Ok(())
}
