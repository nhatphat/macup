use crate::config::{find_config_file, remove_packages_from_config};
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

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
    let result = remove_packages_from_config(&config_file, manager, &packages)?;

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
