use crate::config::Config;
use crate::executor::{ExecutionPlan, SectionType};
use crate::managers::{
    brew::BrewManager, cargo_manager::CargoManager, install::InstallManager, mas::MasManager,
    npm::NpmManager, Manager,
};
use crate::system::SystemManager;
use anyhow::{bail, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

pub fn apply_plan(config: &Config, plan: &ExecutionPlan, dry_run: bool) -> Result<()> {
    let max_parallel = config.settings.max_parallel;

    println!("{}", "=".repeat(50).bright_blue());
    println!("{}", "Starting macup apply".bright_blue().bold());
    println!("{}", "=".repeat(50).bright_blue());
    println!();

    if dry_run {
        println!("{}", "[DRY RUN MODE]".yellow().bold());
        println!();
    }

    for phase in &plan.phases {
        match &phase.section_type {
            SectionType::Managers => {
                println!(
                    "{}",
                    format!("📦 Checking package managers...")
                        .bright_cyan()
                        .bold()
                );

                // Check and install required managers
                for manager_name in &config.managers.required {
                    check_and_install_manager(manager_name, true, dry_run)?;
                }

                // Check optional managers (don't auto-install)
                for manager_name in &config.managers.optional {
                    check_and_install_manager(manager_name, false, dry_run)?;
                }

                println!();
            }

            SectionType::Install => {
                if let Some(install_config) = &config.install {
                    println!(
                        "{}",
                        format!("🔧 Running install scripts...")
                            .bright_cyan()
                            .bold()
                    );

                    if dry_run {
                        for script in &install_config.scripts {
                            println!("  → Would run: {}", script.name);
                        }
                    } else {
                        let install_mgr = InstallManager::new();
                        install_mgr.apply_scripts(&install_config.scripts)?;
                    }

                    println!();
                }
            }

            SectionType::Brew => {
                if let Some(brew_config) = &config.brew {
                    println!(
                        "{}",
                        format!("🍺 Installing Homebrew packages...")
                            .bright_cyan()
                            .bold()
                    );

                    if dry_run {
                        println!("  Taps: {:?}", brew_config.taps);
                        println!("  Formulae: {:?}", brew_config.formulae);
                        println!("  Casks: {:?}", brew_config.casks);
                    } else {
                        let brew = BrewManager::new(max_parallel);

                        if !brew_config.taps.is_empty() {
                            let result = brew.add_taps(&brew_config.taps)?;
                            print_result("Taps", &result);
                        }

                        if !brew_config.formulae.is_empty() {
                            let result = brew.install_formulae(&brew_config.formulae)?;
                            print_result("Formulae", &result);
                        }

                        if !brew_config.casks.is_empty() {
                            let result = brew.install_casks(&brew_config.casks)?;
                            print_result("Casks", &result);
                        }
                    }

                    println!();
                }
            }

            SectionType::Mas => {
                if let Some(mas_config) = &config.mas {
                    println!(
                        "{}",
                        format!("📱 Installing Mac App Store apps...")
                            .bright_cyan()
                            .bold()
                    );

                    if dry_run {
                        for app in &mas_config.apps {
                            println!("  → Would install: {} ({})", app.name, app.id);
                        }
                    } else {
                        let mas = MasManager::new(max_parallel);
                        let app_ids: Vec<String> = mas_config
                            .apps
                            .iter()
                            .map(|app| app.id.to_string())
                            .collect();

                        let result = mas.install_packages(&app_ids)?;
                        print_result("Apps", &result);
                    }

                    println!();
                }
            }

            SectionType::Npm => {
                if let Some(npm_config) = &config.npm {
                    println!(
                        "{}",
                        format!("📦 Installing npm packages...")
                            .bright_cyan()
                            .bold()
                    );

                    if dry_run {
                        println!("  Global packages: {:?}", npm_config.global);
                    } else {
                        let npm = NpmManager::new(max_parallel);
                        let result = npm.install_packages(&npm_config.global)?;
                        print_result("NPM packages", &result);
                    }

                    println!();
                }
            }

            SectionType::Cargo => {
                if let Some(cargo_config) = &config.cargo {
                    println!(
                        "{}",
                        format!("🦀 Installing cargo packages...")
                            .bright_cyan()
                            .bold()
                    );

                    if dry_run {
                        println!("  Packages: {:?}", cargo_config.packages);
                    } else {
                        let cargo_mgr = CargoManager::new(max_parallel);
                        let result = cargo_mgr.install_packages(&cargo_config.packages)?;
                        print_result("Cargo packages", &result);
                    }

                    println!();
                }
            }

            SectionType::System => {
                if let Some(system_config) = &config.system {
                    println!(
                        "{}",
                        format!("⚙️  Applying system settings...")
                            .bright_cyan()
                            .bold()
                    );

                    if dry_run {
                        for cmd in &system_config.commands {
                            println!("  → Would run: {}", cmd);
                        }
                    } else {
                        let system = SystemManager::new();
                        system.apply_commands(&system_config.commands)?;
                    }

                    println!();
                }
            }
        }
    }

    println!("{}", "=".repeat(50).bright_green());
    println!("{}", "✓ macup apply completed!".bright_green().bold());
    println!("{}", "=".repeat(50).bright_green());

    Ok(())
}

fn check_and_install_manager(name: &str, is_required: bool, dry_run: bool) -> Result<()> {
    let exists = crate::utils::command_exists(name);

    if exists {
        println!("  ✓ {} is installed", name.green());
        return Ok(());
    }

    // Not installed
    println!("  ✗ {} is NOT installed", name.red());

    if !is_required {
        println!("    → Skipping (optional)");
        return Ok(());
    }

    // Required manager → auto-install
    if dry_run {
        println!("    → Would install {}", name);
        return Ok(());
    }

    println!("  → Installing {}...", name.yellow());

    match name {
        "brew" => {
            let status = Command::new("sh")
                .arg("-c")
                .arg(r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#)
                .status()?;

            if !status.success() {
                bail!("Failed to install Homebrew");
            }

            // Add to PATH for Apple Silicon Macs
            if Path::new("/opt/homebrew/bin/brew").exists() {
                let current_path = std::env::var("PATH").unwrap_or_default();
                std::env::set_var("PATH", format!("/opt/homebrew/bin:{}", current_path));
            }

            println!("  ✓ {} installed", name.green());
        }
        "mas" => {
            // Install mas via brew
            if !crate::utils::command_exists("brew") {
                bail!("mas requires brew, but brew is not installed");
            }

            let status = Command::new("brew")
                .env("HOMEBREW_NO_AUTO_UPDATE", "1")
                .args(["install", "mas"])
                .status()?;

            if !status.success() {
                bail!("Failed to install mas");
            }

            println!("  ✓ {} installed", name.green());
        }
        _ => {
            bail!(
                "{} is required but not installed. Please install manually.",
                name
            );
        }
    }

    Ok(())
}

fn print_result(_label: &str, result: &crate::managers::InstallResult) {
    if !result.success.is_empty() {
        println!(
            "  ✓ {} installed: {}",
            result.success.len(),
            result.success.len()
        );
    }
    if !result.skipped.is_empty() {
        println!("  ⊘ {} skipped (already installed)", result.skipped.len());
    }
    if !result.failed.is_empty() {
        println!("  ✗ {} failed:", result.failed.len());
        for (pkg, err) in &result.failed {
            println!("    - {}: {}", pkg, err);
        }
    }
}
