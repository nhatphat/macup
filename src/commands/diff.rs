use crate::config::{load_config_auto, CargoConfig, MasConfig, NpmConfig};
use crate::managers::{
    brew::BrewManager,
    cargo_manager::CargoManager, // CODEGEN[cargo]: import
    mas::MasManager, // CODEGEN[mas]: import
    npm::NpmManager, // CODEGEN[npm]: import
    // CODEGEN_MARKER: insert_import_here
    Manager,
    ManagerMetadata,
};
use anyhow::Result;
use colored::Colorize;
use rayon::prelude::*;
use std::path::Path;

/// Result of checking packages for a single manager
#[derive(Debug)]
struct DiffResult {
    manager_name: String,
    icon: String,
    display_name: String,
    installed: Vec<String>,
    missing: Vec<String>,
    skipped_reason: Option<String>, // e.g., "npm not installed"
}

/// Summary of all diff results
#[derive(Debug)]
struct DiffSummary {
    results: Vec<DiffResult>,
    total_installed: usize,
    total_missing: usize,
    total_skipped: usize,
}

pub fn run(config_path: Option<&Path>) -> Result<()> {
    // Load config
    let (_config_path, config) = load_config_auto(config_path)?;

    println!("{}", "=".repeat(60).bright_blue());
    println!(
        "{}",
        "macup diff - Checking installed packages"
            .bright_blue()
            .bold()
    );
    println!("{}", "=".repeat(60).bright_blue());
    println!();

    // Collect all diff results
    let mut results = Vec::new();

    // Check brew sections (taps, formulae, casks)
    if let Some(brew_config) = &config.brew {
        results.extend(check_brew_sections(brew_config));
    }

    // Check mas
    if let Some(mas_config) = &config.mas {
        if let Some(result) = check_mas_section(mas_config) {
            results.push(result);
        }
    }

    // CODEGEN_START[npm]: check_call
    if let Some(npm_config) = &config.npm {
        if let Some(result) = check_npm_section(npm_config) {
            results.push(result);
        }
    }
    // CODEGEN_END[npm]: check_call

    // CODEGEN_START[cargo]: check_call
    if let Some(cargo_config) = &config.cargo {
        if let Some(result) = check_cargo_section(cargo_config) {
            results.push(result);
        }
    }
    // CODEGEN_END[cargo]: check_call

    // CODEGEN_MARKER: insert_check_call_here

    // Calculate summary
    let summary = calculate_summary(results);

    // Display results
    display_results(&summary);

    Ok(())
}

/// Check brew packages (returns multiple results for taps, formulae, casks)
fn check_brew_sections(config: &crate::config::BrewConfig) -> Vec<DiffResult> {
    let mut results = Vec::new();

    // Check taps
    if !config.taps.is_empty() {
        if let Some(result) = check_brew_taps(&config.taps) {
            results.push(result);
        }
    }

    // Check formulae
    if !config.formulae.is_empty() {
        if let Some(result) = check_brew_formulae(&config.formulae) {
            results.push(result);
        }
    }

    // Check casks
    if !config.casks.is_empty() {
        if let Some(result) = check_brew_casks(&config.casks) {
            results.push(result);
        }
    }

    results
}

/// Check brew taps
fn check_brew_taps(taps: &[String]) -> Option<DiffResult> {
    if taps.is_empty() {
        return None;
    }

    // Check if brew is installed
    if !crate::utils::command_exists("brew") {
        return Some(DiffResult {
            manager_name: "brew-taps".to_string(),
            icon: "🍺".to_string(),
            display_name: "Homebrew Taps".to_string(),
            installed: vec![],
            missing: vec![],
            skipped_reason: Some("brew not installed".to_string()),
        });
    }

    // Get list of installed taps
    let brew = BrewManager::new(1);
    let installed_taps = brew.list_taps().unwrap_or_default();

    // Check each tap in parallel
    let tap_results: Vec<_> = taps
        .par_iter()
        .map(|tap| {
            let is_installed = installed_taps.contains(tap);
            (tap.clone(), is_installed)
        })
        .collect();

    let mut installed = vec![];
    let mut missing = vec![];

    for (tap, is_installed) in tap_results {
        if is_installed {
            installed.push(tap);
        } else {
            missing.push(tap);
        }
    }

    Some(DiffResult {
        manager_name: "brew-taps".to_string(),
        icon: "🍺".to_string(),
        display_name: "Homebrew Taps".to_string(),
        installed,
        missing,
        skipped_reason: None,
    })
}

/// Check brew formulae
fn check_brew_formulae(formulae: &[String]) -> Option<DiffResult> {
    if formulae.is_empty() {
        return None;
    }

    // Check if brew is installed
    if !crate::utils::command_exists("brew") {
        return Some(DiffResult {
            manager_name: "brew-formulae".to_string(),
            icon: "🍺".to_string(),
            display_name: "Homebrew Formulae".to_string(),
            installed: vec![],
            missing: vec![],
            skipped_reason: Some("brew not installed".to_string()),
        });
    }

    // Get list of installed formulae
    let brew = BrewManager::new(1);
    let installed_formulae = brew.list_formulae().unwrap_or_default();

    // Check each formula in parallel
    let formula_results: Vec<_> = formulae
        .par_iter()
        .map(|formula| {
            let is_installed = installed_formulae.contains(formula);
            (formula.clone(), is_installed)
        })
        .collect();

    let mut installed = vec![];
    let mut missing = vec![];

    for (formula, is_installed) in formula_results {
        if is_installed {
            installed.push(formula);
        } else {
            missing.push(formula);
        }
    }

    Some(DiffResult {
        manager_name: "brew-formulae".to_string(),
        icon: "🍺".to_string(),
        display_name: "Homebrew Formulae".to_string(),
        installed,
        missing,
        skipped_reason: None,
    })
}

/// Check brew casks
fn check_brew_casks(casks: &[String]) -> Option<DiffResult> {
    if casks.is_empty() {
        return None;
    }

    // Check if brew is installed
    if !crate::utils::command_exists("brew") {
        return Some(DiffResult {
            manager_name: "brew-casks".to_string(),
            icon: "📦".to_string(),
            display_name: "Homebrew Casks".to_string(),
            installed: vec![],
            missing: vec![],
            skipped_reason: Some("brew not installed".to_string()),
        });
    }

    // Get list of installed casks
    let brew = BrewManager::new(1);
    let installed_casks = brew.list_casks().unwrap_or_default();

    // Check each cask in parallel
    let cask_results: Vec<_> = casks
        .par_iter()
        .map(|cask| {
            let is_installed = installed_casks.contains(cask);
            (cask.clone(), is_installed)
        })
        .collect();

    let mut installed = vec![];
    let mut missing = vec![];

    for (cask, is_installed) in cask_results {
        if is_installed {
            installed.push(cask);
        } else {
            missing.push(cask);
        }
    }

    Some(DiffResult {
        manager_name: "brew-casks".to_string(),
        icon: "📦".to_string(),
        display_name: "Homebrew Casks".to_string(),
        installed,
        missing,
        skipped_reason: None,
    })
}

/// Check mas packages
fn check_mas_section(config: &MasConfig) -> Option<DiffResult> {
    if config.apps.is_empty() {
        return None;
    }

    let meta = ManagerMetadata::get_by_name("mas").unwrap();

    // Check if mas is installed
    if !crate::utils::command_exists(meta.runtime_command) {
        return Some(DiffResult {
            manager_name: meta.name.to_string(),
            icon: meta.icon.to_string(),
            display_name: meta.display_name.to_string(),
            installed: vec![],
            missing: vec![],
            skipped_reason: Some(format!("{} not installed", meta.runtime_command)),
        });
    }

    // Check each app in parallel
    let mas_mgr = MasManager::new(1);
    let app_results: Vec<_> = config
        .apps
        .par_iter()
        .map(|app| {
            let display = format!("{} ({})", app.name, app.id);
            let is_installed = mas_mgr
                .is_package_installed(&app.id.to_string())
                .unwrap_or(false);
            (display, is_installed)
        })
        .collect();

    let mut installed = vec![];
    let mut missing = vec![];

    for (display, is_installed) in app_results {
        if is_installed {
            installed.push(display);
        } else {
            missing.push(display);
        }
    }

    Some(DiffResult {
        manager_name: meta.name.to_string(),
        icon: meta.icon.to_string(),
        display_name: meta.display_name.to_string(),
        installed,
        missing,
        skipped_reason: None,
    })
}

// CODEGEN_START[npm]: check_function
/// Check Npm packages
fn check_npm_section(config: &NpmConfig) -> Option<DiffResult> {
    if config.global.is_empty() {
        return None;
    }

    let meta = ManagerMetadata::get_by_name("npm").unwrap();

    // Check if runtime is installed
    if !crate::utils::command_exists(meta.runtime_command) {
        return Some(DiffResult {
            manager_name: meta.name.to_string(),
            icon: meta.icon.to_string(),
            display_name: meta.display_name.to_string(),
            installed: vec![],
            missing: vec![],
            skipped_reason: Some(format!("{} not installed", meta.runtime_command)),
        });
    }

    // Check each package in parallel
    let mgr = NpmManager::new(1);
    let pkg_results: Vec<_> = config
        .global
        .par_iter()
        .map(|pkg| {
            // Parse package:binary format - show only package name
            let (pkg_name, _) = parse_package_name(pkg);
            let is_installed = mgr.is_package_installed(pkg).unwrap_or(false);
            (pkg_name.to_string(), is_installed)
        })
        .collect();

    let mut installed = vec![];
    let mut missing = vec![];

    for (pkg, is_installed) in pkg_results {
        if is_installed {
            installed.push(pkg);
        } else {
            missing.push(pkg);
        }
    }

    Some(DiffResult {
        manager_name: meta.name.to_string(),
        icon: meta.icon.to_string(),
        display_name: meta.display_name.to_string(),
        installed,
        missing,
        skipped_reason: None,
    })
}
// CODEGEN_END[npm]: check_function

// CODEGEN_START[cargo]: check_function
/// Check Cargo packages
fn check_cargo_section(config: &CargoConfig) -> Option<DiffResult> {
    if config.packages.is_empty() {
        return None;
    }

    let meta = ManagerMetadata::get_by_name("cargo").unwrap();

    // Check if runtime is installed
    if !crate::utils::command_exists(meta.runtime_command) {
        return Some(DiffResult {
            manager_name: meta.name.to_string(),
            icon: meta.icon.to_string(),
            display_name: meta.display_name.to_string(),
            installed: vec![],
            missing: vec![],
            skipped_reason: Some(format!("{} not installed", meta.runtime_command)),
        });
    }

    // Check each package in parallel
    let mgr = CargoManager::new(1);
    let pkg_results: Vec<_> = config
        .packages
        .par_iter()
        .map(|pkg| {
            // Parse package:binary format - show only package name
            let (pkg_name, _) = parse_package_name(pkg);
            let is_installed = mgr.is_package_installed(pkg).unwrap_or(false);
            (pkg_name.to_string(), is_installed)
        })
        .collect();

    let mut installed = vec![];
    let mut missing = vec![];

    for (pkg, is_installed) in pkg_results {
        if is_installed {
            installed.push(pkg);
        } else {
            missing.push(pkg);
        }
    }

    Some(DiffResult {
        manager_name: meta.name.to_string(),
        icon: meta.icon.to_string(),
        display_name: meta.display_name.to_string(),
        installed,
        missing,
        skipped_reason: None,
    })
}
// CODEGEN_END[cargo]: check_function

// CODEGEN_MARKER: insert_check_function_here

/// Parse package:binary format
fn parse_package_name(input: &str) -> (&str, &str) {
    if let Some((pkg, bin)) = input.split_once(':') {
        (pkg.trim(), bin.trim())
    } else {
        (input.trim(), input.trim())
    }
}

/// Calculate summary from all results
fn calculate_summary(results: Vec<DiffResult>) -> DiffSummary {
    let mut total_installed = 0;
    let mut total_missing = 0;
    let mut total_skipped = 0;

    for result in &results {
        if result.skipped_reason.is_some() {
            total_skipped += 1;
        } else {
            total_installed += result.installed.len();
            total_missing += result.missing.len();
        }
    }

    DiffSummary {
        results,
        total_installed,
        total_missing,
        total_skipped,
    }
}

/// Display diff results with colored output
fn display_results(summary: &DiffSummary) {
    // Display each manager's results
    for result in &summary.results {
        // Show manager header
        println!(
            "{} {}",
            result.icon,
            result.display_name.bright_cyan().bold()
        );

        // Check if skipped
        if let Some(reason) = &result.skipped_reason {
            println!("  {} {}", "⚠️".yellow(), reason.yellow());
            println!();
            continue;
        }

        // Show installed packages
        for pkg in &result.installed {
            println!("  {} {}", "✓".green(), pkg.green());
        }

        // Show missing packages
        for pkg in &result.missing {
            println!("  {} {}", "❌".red(), pkg.red());
        }

        // Show summary for this manager
        let total = result.installed.len() + result.missing.len();
        if total > 0 {
            println!(
                "  {}: {}/{}",
                "Summary".dimmed(),
                result.installed.len(),
                total
            );
        }

        println!();
    }

    // Overall summary
    println!("{}", "=".repeat(60).bright_blue());
    println!("{}", "Overall Summary".bright_blue().bold());
    println!("{}", "=".repeat(60).bright_blue());

    if summary.total_installed > 0 {
        println!("  {} Installed: {}", "✓".green(), summary.total_installed);
    }
    if summary.total_missing > 0 {
        println!("  {} Missing: {}", "❌".red(), summary.total_missing);
    }
    if summary.total_skipped > 0 {
        println!(
            "  {} Skipped: {} manager(s)",
            "⊘".yellow(),
            summary.total_skipped
        );
    }

    // No packages configured
    if summary.results.is_empty() {
        println!("  {}", "No packages configured".dimmed());
    }

    println!();

    // Show suggestion if there are missing packages
    if summary.total_missing > 0 {
        println!(
            "{}",
            "Run 'macup apply' to install missing packages.".bright_yellow()
        );
        println!();
    }
}
