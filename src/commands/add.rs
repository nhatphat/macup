use crate::config::{add_packages_to_config, find_config_file, load_config};
use crate::managers::{
    brew::BrewManager,
    cargo_manager::CargoManager, // CODEGEN[cargo]: import
    mas::MasManager,             // CODEGEN[mas]: import
    npm::NpmManager,             // CODEGEN[npm]: import
    // CODEGEN_MARKER: insert_manager_import_here
    Manager,
    ManagerMetadata,
    PACKAGE_MANAGERS,
};
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

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

    // Get manager instance - check registry first, then special cases
    let mgr: Box<dyn Manager> =
        if let Some(meta) = ManagerMetadata::get_by_name(manager) {
            // Dynamic manager from registry
            match meta.name {
                // CODEGEN_START[mas]: match_arm
                "mas" => Box::new(MasManager::new(max_parallel)),
                // CODEGEN_END[mas]: match_arm
                // CODEGEN_START[npm]: match_arm
                "npm" => Box::new(NpmManager::new(max_parallel)),
                // CODEGEN_END[npm]: match_arm
                // CODEGEN_START[cargo]: match_arm
                "cargo" => Box::new(CargoManager::new(max_parallel)),
                // CODEGEN_END[cargo]: match_arm
                // CODEGEN_MARKER: insert_manager_match_arm_here
                _ => {
                    anyhow::bail!(
                "Manager '{}' found in registry but not implemented in 'macup add' command.\n\
                 Use 'macup apply' instead, or add {}Manager to src/commands/add.rs",
                manager,
                manager.chars().next().unwrap().to_uppercase().collect::<String>() + &manager[1..]
            )
                }
            }
        } else {
            // Special cases not in registry
            match manager {
                "brew" => Box::new(BrewManager::new(max_parallel)),
                "cask" => Box::new(BrewManager::new(max_parallel)),
                _ => {
                    // Show available managers from registry
                    let available: Vec<_> = PACKAGE_MANAGERS.iter().map(|m| m.name).collect();
                    anyhow::bail!(
                        "Unknown manager: '{}'. Valid: brew, cask, {}",
                        manager,
                        available.join(", ")
                    )
                }
            }
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
        add_packages_to_config(&config_file, manager, &to_add)?;
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
