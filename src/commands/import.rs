use crate::config::{load_config_auto, Config};
use anyhow::{Context, Result};
use colored::Colorize;
use inquire::MultiSelect;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use toml_edit::{value, Array, DocumentMut};

/// Represents a package manager type
#[derive(Debug, Clone, PartialEq)]
enum PackageManager {
    BrewFormula,
    BrewCask,
    Npm,
    Cargo,
    Mas,
    Pipx,
}

/// Extra data for certain package types
#[derive(Debug, Clone)]
enum ExtraData {
    MasApp { id: u64 },
}

/// A scanned package from the system
#[derive(Debug, Clone)]
struct ScannedPackage {
    name: String,
    manager: PackageManager,
    manager_section: String,
    extra_data: Option<ExtraData>,
    is_existing: bool,
}

/// Main entry point for import command
pub fn run(config_path: Option<&Path>) -> Result<()> {
    println!("{}", "=".repeat(60).bright_blue());
    println!(
        "{}",
        "macup import - Scan system packages".bright_blue().bold()
    );
    println!("{}", "=".repeat(60).bright_blue());
    println!();

    // 1. Scan system
    println!("{}", "Scanning system packages...".cyan());
    let mut packages = scan_system()?;

    if packages.is_empty() {
        println!("{}", "No packages found on system.".yellow());
        return Ok(());
    }

    println!("  {} Found {} packages", "✓".green(), packages.len());
    println!();

    // 2. Load config and detect existing
    let (resolved_path, config) = load_config_auto(config_path)?;
    detect_existing(&mut packages, &config)?;

    // 3. Interactive selection
    let selected = interactive_select(packages)?;

    if selected.is_empty() {
        println!("{}", "No packages selected.".yellow());
        return Ok(());
    }

    // 4. Auto-detect taps
    let taps = collect_required_taps(&selected);

    // 5. Generate preview
    println!();
    println!("{}", "=".repeat(60).bright_blue());
    println!("{}", "Preview - Will add to config:".bright_blue().bold());
    println!("{}", "=".repeat(60).bright_blue());
    println!();

    let preview = generate_toml_preview(&selected, &taps)?;
    println!("{}", preview);

    // 6. Confirm
    let confirmed = inquire::Confirm::new("Add these packages to macup.toml?")
        .with_default(true)
        .prompt()?;

    if !confirmed {
        println!("{}", "Import cancelled.".yellow());
        return Ok(());
    }

    // 7. Merge to config
    println!();
    println!("{}", "Writing to config...".cyan());
    merge_to_config(&resolved_path, &selected, &taps)?;

    println!("{}", "=".repeat(60).bright_green());
    println!(
        "{}",
        "✅ Import completed successfully!".bright_green().bold()
    );
    println!("{}", "=".repeat(60).bright_green());
    println!();
    println!(
        "Added {} packages to {}",
        selected.len(),
        resolved_path.display()
    );
    println!();
    println!("{}", "Next steps:".bold());
    println!("  • Run {} to verify changes", "macup diff".cyan());
    println!("  • Run {} to apply on a new machine", "macup apply".cyan());
    println!();

    Ok(())
}

/// Scan all package managers on the system
fn scan_system() -> Result<Vec<ScannedPackage>> {
    let mut packages = Vec::new();

    // Scan each manager in parallel
    let results: Vec<Result<Vec<ScannedPackage>>> = vec![
        scan_brew_formulae(),
        scan_brew_casks(),
        scan_npm_global(),
        scan_cargo(),
        scan_mas(),
        scan_pipx(),
    ]
    .into_par_iter()
    .map(|f| f)
    .collect();

    for result in results {
        packages.extend(result?);
    }

    Ok(packages)
}

/// Scan Homebrew formulae
fn scan_brew_formulae() -> Result<Vec<ScannedPackage>> {
    if !crate::utils::command_exists("brew") {
        return Ok(vec![]);
    }

    let output = Command::new("brew")
        .args(&["list", "--formula"])
        .output()
        .context("Failed to run brew list")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let formulae: Vec<_> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            // Skip tap detection for now (too slow)
            // User can manually add taps if needed
            ScannedPackage {
                name: line.to_string(),
                manager: PackageManager::BrewFormula,
                manager_section: "brew-formulae".to_string(),
                extra_data: None,
                is_existing: false,
            }
        })
        .collect();

    Ok(formulae)
}

/// Scan Homebrew casks
fn scan_brew_casks() -> Result<Vec<ScannedPackage>> {
    if !crate::utils::command_exists("brew") {
        return Ok(vec![]);
    }

    let output = Command::new("brew")
        .args(&["list", "--cask"])
        .output()
        .context("Failed to run brew list --cask")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let casks: Vec<_> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            // Skip tap detection for now (too slow)
            ScannedPackage {
                name: line.to_string(),
                manager: PackageManager::BrewCask,
                manager_section: "brew-casks".to_string(),
                extra_data: None,
                is_existing: false,
            }
        })
        .collect();

    Ok(casks)
}

/// Scan npm global packages
fn scan_npm_global() -> Result<Vec<ScannedPackage>> {
    if !crate::utils::command_exists("npm") {
        return Ok(vec![]);
    }

    let output = Command::new("npm")
        .args(&["list", "-g", "--depth=0", "--json"])
        .output()
        .context("Failed to run npm list")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let deps = match json["dependencies"].as_object() {
        Some(d) => d,
        None => return Ok(vec![]),
    };

    let packages: Vec<_> = deps
        .keys()
        .filter(|&name| name != "npm" && name != "corepack")
        .map(|name| ScannedPackage {
            name: name.clone(),
            manager: PackageManager::Npm,
            manager_section: "npm".to_string(),
            extra_data: None,
            is_existing: false,
        })
        .collect();

    Ok(packages)
}

/// Scan cargo installed packages
fn scan_cargo() -> Result<Vec<ScannedPackage>> {
    if !crate::utils::command_exists("cargo") {
        return Ok(vec![]);
    }

    let output = Command::new("cargo")
        .args(&["install", "--list"])
        .output()
        .context("Failed to run cargo install --list")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let packages: Vec<_> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_whitespace().next())
        .map(|name| ScannedPackage {
            name: name.to_string(),
            manager: PackageManager::Cargo,
            manager_section: "cargo".to_string(),
            extra_data: None,
            is_existing: false,
        })
        .collect();

    Ok(packages)
}

/// Scan Mac App Store apps
fn scan_mas() -> Result<Vec<ScannedPackage>> {
    if !crate::utils::command_exists("mas") {
        return Ok(vec![]);
    }

    let output = Command::new("mas")
        .arg("list")
        .output()
        .context("Failed to run mas list")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let apps: Vec<_> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            // Format: "497799835 Xcode (16.2)"
            let parts: Vec<_> = line.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let id = parts[0].parse::<u64>().ok()?;
                let name = parts[1].split('(').next()?.trim();
                Some(ScannedPackage {
                    name: name.to_string(),
                    manager: PackageManager::Mas,
                    manager_section: "mas".to_string(),
                    extra_data: Some(ExtraData::MasApp { id }),
                    is_existing: false,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(apps)
}

/// Scan pipx packages
fn scan_pipx() -> Result<Vec<ScannedPackage>> {
    if !crate::utils::command_exists("pipx") {
        return Ok(vec![]);
    }

    let output = Command::new("pipx")
        .args(&["list", "--short"])
        .output()
        .context("Failed to run pipx list")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let packages: Vec<_> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            // Format: "poetry 2.1.3"
            let name = line.split_whitespace().next().unwrap_or(line);
            ScannedPackage {
                name: name.to_string(),
                manager: PackageManager::Pipx,
                manager_section: "pipx".to_string(),
                extra_data: None,
                is_existing: false,
            }
        })
        .collect();

    Ok(packages)
}

/// Detect which packages already exist in config
fn detect_existing(packages: &mut [ScannedPackage], config: &Config) -> Result<()> {
    for pkg in packages.iter_mut() {
        let exists = match pkg.manager {
            PackageManager::BrewFormula => config
                .brew
                .as_ref()
                .map(|b| b.formulae.contains(&pkg.name))
                .unwrap_or(false),
            PackageManager::BrewCask => config
                .brew
                .as_ref()
                .map(|b| b.casks.contains(&pkg.name))
                .unwrap_or(false),
            PackageManager::Npm => config
                .npm
                .as_ref()
                .map(|n| n.global.contains(&pkg.name))
                .unwrap_or(false),
            PackageManager::Cargo => config
                .cargo
                .as_ref()
                .map(|c| c.packages.contains(&pkg.name))
                .unwrap_or(false),
            PackageManager::Mas => {
                if let Some(ExtraData::MasApp { id }) = pkg.extra_data {
                    config
                        .mas
                        .as_ref()
                        .map(|m| m.apps.iter().any(|app| app.id == id))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            PackageManager::Pipx => false,
        };

        pkg.is_existing = exists;
    }

    Ok(())
}

/// Interactive selection UI
fn interactive_select(packages: Vec<ScannedPackage>) -> Result<Vec<ScannedPackage>> {
    if packages.is_empty() {
        return Ok(vec![]);
    }

    println!("{}", "=".repeat(60).bright_blue());
    println!("{}", "Select packages to import".bright_blue().bold());
    println!("{}", "=".repeat(60).bright_blue());
    println!();
    println!("{}", "Controls:".bold());
    println!("  {} Navigate", "↑/↓".cyan());
    println!("  {} Toggle selection", "Space".cyan());
    println!("  {} Confirm selection", "Enter".cyan());
    println!();

    // Build options for display, grouped by section
    let mut options = Vec::new();
    let mut pkg_map = Vec::new();

    for pkg in &packages {
        // Format package name
        let display = if pkg.is_existing {
            format!(
                "{} {} {}",
                section_icon(&pkg.manager_section),
                pkg.name,
                "[existing]".dimmed()
            )
        } else {
            format!("{} {}", section_icon(&pkg.manager_section), pkg.name)
        };

        options.push(display);
        pkg_map.push(pkg.clone());
    }

    let selections = MultiSelect::new("Select packages:", options).prompt()?;

    // Map selections back to packages
    let selected: Vec<_> = selections
        .into_iter()
        .filter_map(|display| {
            // Find pkg by matching display string
            pkg_map
                .iter()
                .position(|p| display.contains(&p.name))
                .map(|idx| pkg_map[idx].clone())
        })
        .collect();

    Ok(selected)
}

/// Get icon for section
fn section_icon(section: &str) -> &'static str {
    match section {
        "brew-formulae" => "🍺",
        "brew-casks" => "📦",
        "npm" => "📦",
        "cargo" => "🦀",
        "mas" => "📱",
        "pipx" => "🐍",
        _ => "📦",
    }
}

/// Collect required taps from selected packages
fn collect_required_taps(_packages: &[ScannedPackage]) -> Vec<String> {
    // Tap auto-detection disabled for performance
    // Users can manually add taps if needed
    Vec::new()
}

/// Generate TOML preview
fn generate_toml_preview(packages: &[ScannedPackage], taps: &[String]) -> Result<String> {
    let mut preview = String::new();

    // Group by manager
    let mut brew_formulae = Vec::new();
    let mut brew_casks = Vec::new();
    let mut npm_packages = Vec::new();
    let mut cargo_packages = Vec::new();
    let mut mas_apps = Vec::new();
    let mut pipx_packages = Vec::new();

    for pkg in packages {
        match pkg.manager {
            PackageManager::BrewFormula => brew_formulae.push(pkg.name.clone()),
            PackageManager::BrewCask => brew_casks.push(pkg.name.clone()),
            PackageManager::Npm => npm_packages.push(pkg.name.clone()),
            PackageManager::Cargo => cargo_packages.push(pkg.name.clone()),
            PackageManager::Mas => {
                if let Some(ExtraData::MasApp { id }) = pkg.extra_data {
                    mas_apps.push((pkg.name.clone(), id));
                }
            }
            PackageManager::Pipx => pipx_packages.push(pkg.name.clone()),
        }
    }

    // Generate brew section
    if !taps.is_empty() || !brew_formulae.is_empty() || !brew_casks.is_empty() {
        preview.push_str("[brew]\n");

        if !taps.is_empty() {
            preview.push_str("taps = [\n");
            for tap in taps {
                preview.push_str(&format!("    \"{}\",\n", tap));
            }
            preview.push_str("]\n\n");
        }

        if !brew_formulae.is_empty() {
            preview.push_str("formulae = [\n");
            for formula in &brew_formulae {
                preview.push_str(&format!("    \"{}\",\n", formula));
            }
            preview.push_str("]\n\n");
        }

        if !brew_casks.is_empty() {
            preview.push_str("casks = [\n");
            for cask in &brew_casks {
                preview.push_str(&format!("    \"{}\",\n", cask));
            }
            preview.push_str("]\n");
        }
    }

    if !mas_apps.is_empty() {
        if !preview.is_empty() {
            preview.push('\n');
        }
        preview.push_str("[mas]\n");
        preview.push_str("apps = [\n");
        for (name, id) in &mas_apps {
            preview.push_str(&format!("    {{ name = \"{}\", id = {} }},\n", name, id));
        }
        preview.push_str("]\n");
    }

    if !npm_packages.is_empty() {
        if !preview.is_empty() {
            preview.push('\n');
        }
        preview.push_str("[npm]\n");
        preview.push_str("global = [\n");
        for pkg in &npm_packages {
            preview.push_str(&format!("    \"{}\",\n", pkg));
        }
        preview.push_str("]\n");
    }

    if !cargo_packages.is_empty() {
        if !preview.is_empty() {
            preview.push('\n');
        }
        preview.push_str("[cargo]\n");
        preview.push_str("packages = [\n");
        for pkg in &cargo_packages {
            preview.push_str(&format!("    \"{}\",\n", pkg));
        }
        preview.push_str("]\n");
    }

    if !pipx_packages.is_empty() {
        if !preview.is_empty() {
            preview.push('\n');
        }
        preview.push_str("# Note: pipx is not a built-in manager yet\n");
        preview.push_str("# Add support with: macup new manager pipx ...\n");
        preview.push_str("\n[pipx]\n");
        preview.push_str("packages = [\n");
        for pkg in &pipx_packages {
            preview.push_str(&format!("    \"{}\",\n", pkg));
        }
        preview.push_str("]\n");
    }

    Ok(preview)
}

/// Merge selected packages into config file
fn merge_to_config(config_path: &Path, packages: &[ScannedPackage], taps: &[String]) -> Result<()> {
    // Read existing config
    let content = fs::read_to_string(config_path).context("Failed to read config file")?;
    let mut doc = content
        .parse::<DocumentMut>()
        .context("Failed to parse TOML")?;

    // Group packages by type
    let mut brew_formulae = Vec::new();
    let mut brew_casks = Vec::new();
    let mut npm_packages = Vec::new();
    let mut cargo_packages = Vec::new();
    let mut mas_apps = Vec::new();
    let mut pipx_packages = Vec::new();

    for pkg in packages {
        match pkg.manager {
            PackageManager::BrewFormula => brew_formulae.push(pkg.name.clone()),
            PackageManager::BrewCask => brew_casks.push(pkg.name.clone()),
            PackageManager::Npm => npm_packages.push(pkg.name.clone()),
            PackageManager::Cargo => cargo_packages.push(pkg.name.clone()),
            PackageManager::Mas => {
                if let Some(ExtraData::MasApp { id }) = pkg.extra_data {
                    mas_apps.push((pkg.name.clone(), id));
                }
            }
            PackageManager::Pipx => pipx_packages.push(pkg.name.clone()),
        }
    }

    // Ensure [brew] section exists if needed
    if !taps.is_empty() || !brew_formulae.is_empty() || !brew_casks.is_empty() {
        if !doc.contains_key("brew") {
            doc["brew"] = toml_edit::table();
        }

        // Add taps
        if !taps.is_empty() {
            let mut tap_array = doc["brew"]["taps"]
                .as_array()
                .cloned()
                .unwrap_or_else(Array::new);

            for tap in taps {
                if !array_contains_str(&tap_array, tap) {
                    tap_array.push(tap.as_str());
                }
            }
            doc["brew"]["taps"] = value(tap_array);
        }

        // Merge formulae
        if !brew_formulae.is_empty() {
            let mut array = doc["brew"]["formulae"]
                .as_array()
                .cloned()
                .unwrap_or_else(Array::new);

            for formula in &brew_formulae {
                if !array_contains_str(&array, formula) {
                    array.push(formula.as_str());
                }
            }
            doc["brew"]["formulae"] = value(array);
        }

        // Merge casks
        if !brew_casks.is_empty() {
            let mut array = doc["brew"]["casks"]
                .as_array()
                .cloned()
                .unwrap_or_else(Array::new);

            for cask in &brew_casks {
                if !array_contains_str(&array, cask) {
                    array.push(cask.as_str());
                }
            }
            doc["brew"]["casks"] = value(array);
        }
    }

    // Merge npm packages
    if !npm_packages.is_empty() {
        if !doc.contains_key("npm") {
            doc["npm"] = toml_edit::table();
        }

        let mut array = doc["npm"]["global"]
            .as_array()
            .cloned()
            .unwrap_or_else(Array::new);

        for pkg in &npm_packages {
            if !array_contains_str(&array, pkg) {
                array.push(pkg.as_str());
            }
        }
        doc["npm"]["global"] = value(array);
    }

    // Merge cargo packages
    if !cargo_packages.is_empty() {
        if !doc.contains_key("cargo") {
            doc["cargo"] = toml_edit::table();
        }

        let mut array = doc["cargo"]["packages"]
            .as_array()
            .cloned()
            .unwrap_or_else(Array::new);

        for pkg in &cargo_packages {
            if !array_contains_str(&array, pkg) {
                array.push(pkg.as_str());
            }
        }
        doc["cargo"]["packages"] = value(array);
    }

    // Merge MAS apps
    if !mas_apps.is_empty() {
        if !doc.contains_key("mas") {
            doc["mas"] = toml_edit::table();
        }

        let mut apps_array = doc["mas"]["apps"]
            .as_array_of_tables()
            .cloned()
            .unwrap_or_else(toml_edit::ArrayOfTables::new);

        for (name, id) in &mas_apps {
            // Check if app already exists by ID
            let exists = apps_array.iter().any(|app| {
                app.get("id")
                    .and_then(|v| v.as_integer())
                    .map(|i| i == *id as i64)
                    .unwrap_or(false)
            });

            if !exists {
                let mut table = toml_edit::Table::new();
                table.insert("name", value(name.as_str()));
                table.insert("id", value(*id as i64));
                apps_array.push(table);
            }
        }

        doc["mas"]["apps"] = toml_edit::Item::ArrayOfTables(apps_array);
    }

    // Write pipx as comment if any
    if !pipx_packages.is_empty() {
        // Just add a comment about pipx for now
        // User would need to implement pipx manager first
    }

    // Write back
    fs::write(config_path, doc.to_string()).context("Failed to write config file")?;

    Ok(())
}

/// Check if array contains a string value
fn array_contains_str(array: &Array, item: &str) -> bool {
    array.iter().any(|v| v.as_str() == Some(item))
}
