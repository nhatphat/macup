use crate::config::Config;
use crate::executor::{ExecutionPlan, SectionType};
use crate::managers::{
    brew::BrewManager,
    cargo_manager::CargoManager, // CODEGEN[cargo]: import
    install::InstallManager,
    mas::MasManager, // CODEGEN[mas]: import
    npm::NpmManager, // CODEGEN[npm]: import
    // CODEGEN_MARKER: insert_manager_import_here
    Manager,
    ManagerMetadata,
};
use crate::system::SystemManager;
use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// Tracks execution context and state
#[derive(Debug, Default)]
struct ExecutionContext {
    available_managers: HashSet<String>,
    skipped_phases: Vec<SkippedPhase>,
}

#[derive(Debug)]
struct SkippedPhase {
    name: String,
    reason: String,
}

/// Tracks failures during apply execution
#[derive(Debug, Default)]
struct ApplyErrors {
    manager_failures: Vec<ManagerFailure>,
    package_failures: Vec<PackageFailure>,
}

#[derive(Debug)]
struct ManagerFailure {
    name: String,
    reason: String,
}

#[derive(Debug)]
struct PackageFailure {
    package: String,
    manager: String,
    reason: String,
}

impl ApplyErrors {
    fn has_failures(&self) -> bool {
        !self.manager_failures.is_empty() || !self.package_failures.is_empty()
    }
}

// CODEGEN_START[mas]: handler_function
/// Handler for Mas package manager phase
fn apply_mas_phase(
    config: &Config,
    dry_run: bool,
    max_parallel: usize,
    fail_fast: bool,
    errors: &mut ApplyErrors,
) -> Result<()> {
    let mas_config = match &config.mas {
        Some(cfg) if !cfg.apps.is_empty() => cfg,
        _ => return Ok(()), // No mas config or no apps
    };

    let meta = ManagerMetadata::get_by_name("mas").unwrap();

    println!(
        "{}",
        format!("{} Installing {}...", meta.icon, meta.display_name)
            .bright_cyan()
            .bold()
    );

    // Auto-install mas if not found
    if !crate::utils::command_exists(meta.runtime_command) {
        println!(
            "  ⚠️  {} not found, installing {} via brew...",
            meta.runtime_command.yellow(),
            meta.runtime_name.cyan()
        );

        if dry_run {
            println!("    → Would run: brew install {}", meta.brew_formula);
        } else {
            match install_runtime_via_brew(meta.brew_formula) {
                Ok(_) => {
                    println!("  ✓ {} installed", meta.runtime_name.green());
                }
                Err(e) => {
                    println!("  ❌ Failed to install {}: {}", meta.runtime_name, e);

                    // Record failures for all apps
                    for app in &mas_config.apps {
                        errors.package_failures.push(PackageFailure {
                            package: format!("{} ({})", app.name, app.id),
                            manager: meta.name.to_string(),
                            reason: format!("{} installation failed: {}", meta.runtime_name, e),
                        });
                    }

                    if fail_fast {
                        bail!("Failed to install {}", meta.runtime_name);
                    }

                    println!();
                    return Ok(());
                }
            }
        }
    }

    // Install apps
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

        match mas.install_packages(&app_ids) {
            Ok(result) => {
                print_result("Apps", &result);

                // Track failures
                for (pkg, reason) in &result.failed {
                    errors.package_failures.push(PackageFailure {
                        package: pkg.clone(),
                        manager: meta.name.to_string(),
                        reason: reason.clone(),
                    });
                }
            }
            Err(e) => {
                println!("  ❌ {} installation failed: {}", meta.name, e);

                if fail_fast {
                    bail!("{} installation failed", meta.name);
                }
            }
        }
    }

    println!();
    Ok(())
}
// CODEGEN_END[mas]: handler_function

// CODEGEN_START[npm]: handler_function
/// Handler for Npm package manager phase
fn apply_npm_phase(
    config: &Config,
    dry_run: bool,
    max_parallel: usize,
    fail_fast: bool,
    errors: &mut ApplyErrors,
) -> Result<()> {
    let npm_config = match &config.npm {
        Some(cfg) if !cfg.global.is_empty() => cfg,
        _ => return Ok(()), // No npm config or no packages
    };

    let meta = ManagerMetadata::get_by_name("npm").unwrap();

    println!(
        "{}",
        format!("{} Installing {}...", meta.icon, meta.display_name)
            .bright_cyan()
            .bold()
    );

    // Auto-install node if npm not found
    if !crate::utils::command_exists(meta.runtime_command) {
        println!(
            "  ⚠️  {} not found, installing {} via brew...",
            meta.runtime_command.yellow(),
            meta.runtime_name.cyan()
        );

        if dry_run {
            println!("    → Would run: brew install {}", meta.brew_formula);
        } else {
            match install_runtime_via_brew(meta.brew_formula) {
                Ok(_) => {
                    println!("  ✓ {} installed", meta.runtime_name.green());
                }
                Err(e) => {
                    println!("  ❌ Failed to install {}: {}", meta.runtime_name, e);

                    // Record failures for all packages
                    for pkg in &npm_config.global {
                        errors.package_failures.push(PackageFailure {
                            package: pkg.clone(),
                            manager: meta.name.to_string(),
                            reason: format!("{} installation failed: {}", meta.runtime_name, e),
                        });
                    }

                    if fail_fast {
                        bail!("Failed to install {}", meta.runtime_name);
                    }

                    println!();
                    return Ok(());
                }
            }
        }
    }

    // Install packages
    if dry_run {
        println!("  Global packages: {:?}", npm_config.global);
    } else {
        let npm = NpmManager::new(max_parallel);
        match npm.install_packages(&npm_config.global) {
            Ok(result) => {
                print_result("NPM packages", &result);

                // Track failures
                for (pkg, reason) in &result.failed {
                    errors.package_failures.push(PackageFailure {
                        package: pkg.clone(),
                        manager: meta.name.to_string(),
                        reason: reason.clone(),
                    });
                }
            }
            Err(e) => {
                println!("  ❌ {} installation failed: {}", meta.name, e);

                if fail_fast {
                    bail!("{} installation failed", meta.name);
                }
            }
        }
    }

    println!();
    Ok(())
}
// CODEGEN_END[npm]: handler_function

// CODEGEN_START[cargo]: handler_function
/// Handler for Cargo package manager phase  
fn apply_cargo_phase(
    config: &Config,
    dry_run: bool,
    max_parallel: usize,
    fail_fast: bool,
    errors: &mut ApplyErrors,
) -> Result<()> {
    let cargo_config = match &config.cargo {
        Some(cfg) if !cfg.packages.is_empty() => cfg,
        _ => return Ok(()), // No cargo config or no packages
    };

    let meta = ManagerMetadata::get_by_name("cargo").unwrap();

    println!(
        "{}",
        format!("{} Installing {}...", meta.icon, meta.display_name)
            .bright_cyan()
            .bold()
    );

    // Auto-install rust if cargo not found
    if !crate::utils::command_exists(meta.runtime_command) {
        // Check if rustup exists first
        if crate::utils::command_exists("rustup") {
            println!("  ⚠️  cargo not found, installing via rustup...");

            if !dry_run {
                match Command::new("rustup")
                    .args(["toolchain", "install", "stable"])
                    .status()
                {
                    Ok(status) if status.success() => {
                        println!("  ✓ {} installed", "rust".green());
                    }
                    _ => {
                        println!("  ❌ Failed to install rust via rustup");

                        for pkg in &cargo_config.packages {
                            errors.package_failures.push(PackageFailure {
                                package: pkg.clone(),
                                manager: meta.name.to_string(),
                                reason: "rust installation via rustup failed".to_string(),
                            });
                        }

                        if fail_fast {
                            bail!("Failed to install rust via rustup");
                        }

                        println!();
                        return Ok(());
                    }
                }
            }
        } else {
            println!(
                "  ⚠️  {} not found, installing {} via brew...",
                meta.runtime_command.yellow(),
                meta.runtime_name.cyan()
            );

            if dry_run {
                println!("    → Would run: brew install {}", meta.brew_formula);
            } else {
                match install_runtime_via_brew(meta.brew_formula) {
                    Ok(_) => {
                        println!("  ✓ {} installed", meta.runtime_name.green());
                    }
                    Err(e) => {
                        println!("  ❌ Failed to install {}: {}", meta.runtime_name, e);

                        for pkg in &cargo_config.packages {
                            errors.package_failures.push(PackageFailure {
                                package: pkg.clone(),
                                manager: meta.name.to_string(),
                                reason: format!("{} installation failed: {}", meta.runtime_name, e),
                            });
                        }

                        if fail_fast {
                            bail!("Failed to install {}", meta.runtime_name);
                        }

                        println!();
                        return Ok(());
                    }
                }
            }
        }
    }

    // Install packages
    if dry_run {
        println!("  Packages: {:?}", cargo_config.packages);
    } else {
        let cargo_mgr = CargoManager::new(max_parallel);
        match cargo_mgr.install_packages(&cargo_config.packages) {
            Ok(result) => {
                print_result("Cargo packages", &result);

                // Track failures
                for (pkg, reason) in &result.failed {
                    errors.package_failures.push(PackageFailure {
                        package: pkg.clone(),
                        manager: meta.name.to_string(),
                        reason: reason.clone(),
                    });
                }
            }
            Err(e) => {
                println!("  ❌ {} installation failed: {}", meta.name, e);

                if fail_fast {
                    bail!("{} installation failed", meta.name);
                }
            }
        }
    }

    println!();
    Ok(())
}
// CODEGEN_END[cargo]: handler_function

// CODEGEN_MARKER: insert_handler_function_here

pub fn apply_plan(
    config: &Config,
    plan: &ExecutionPlan,
    dry_run: bool,
    with_system_settings: bool,
) -> Result<()> {
    let max_parallel = config.settings.max_parallel;
    let fail_fast = config.settings.fail_fast;
    let mut errors = ApplyErrors::default();
    let mut ctx = ExecutionContext::default();

    println!("{}", "=".repeat(50).bright_blue());
    println!("{}", "Starting macup apply".bright_blue().bold());
    println!("{}", "=".repeat(50).bright_blue());
    println!();

    if dry_run {
        println!("{}", "[DRY RUN MODE]".yellow().bold());
        println!();
    }

    for phase in &plan.phases {
        // Check if dependencies are satisfied
        if !can_execute_phase(phase, &ctx.available_managers) {
            let missing_deps: Vec<_> = phase
                .depends_on
                .iter()
                .filter(|dep| !ctx.available_managers.contains(*dep))
                .collect();

            let reason = format!(
                "Missing dependencies: {}",
                missing_deps
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            ctx.skipped_phases.push(SkippedPhase {
                name: phase.name.clone(),
                reason: reason.clone(),
            });

            println!(
                "  ⚠️  Skipping {} phase: {}",
                phase.name.yellow(),
                reason.yellow()
            );
            println!();
            continue;
        }

        match &phase.section_type {
            SectionType::Managers => {
                println!(
                    "{}",
                    format!("📦 Checking package managers...")
                        .bright_cyan()
                        .bold()
                );

                // Get required managers (auto-detected)
                let required_managers = config.get_required_managers();

                if required_managers.is_empty() {
                    println!("  (No package managers required)");
                } else {
                    for manager_name in &required_managers {
                        match check_and_install_manager(manager_name, dry_run) {
                            Ok(_) => {
                                // Track successfully installed/available manager
                                ctx.available_managers.insert(manager_name.clone());
                            }
                            Err(e) => {
                                println!("  ❌ Failed to install {}: {}", manager_name.red(), e);

                                errors.manager_failures.push(ManagerFailure {
                                    name: manager_name.clone(),
                                    reason: e.to_string(),
                                });

                                if fail_fast {
                                    bail!("Manager installation failed: {}", manager_name);
                                }
                            }
                        }
                    }
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

            // CODEGEN_START[mas]: match_arm
            SectionType::Mas => {
                apply_mas_phase(config, dry_run, max_parallel, fail_fast, &mut errors)?;
            }
            // CODEGEN_END[mas]: match_arm

            // CODEGEN_START[npm]: match_arm
            SectionType::Npm => {
                apply_npm_phase(config, dry_run, max_parallel, fail_fast, &mut errors)?;
            }
            // CODEGEN_END[npm]: match_arm

            // CODEGEN_START[cargo]: match_arm
            SectionType::Cargo => {
                apply_cargo_phase(config, dry_run, max_parallel, fail_fast, &mut errors)?;
            }
            // CODEGEN_END[cargo]: match_arm

            // CODEGEN_MARKER: insert_section_match_arm_here
            SectionType::System => {
                // Skip system settings unless explicitly requested
                if !with_system_settings {
                    if config.system.is_some() {
                        println!(
                            "{}",
                            "⊘ Skipping system settings (use --with-system-settings to apply)"
                                .yellow()
                        );
                        println!();
                    }
                    continue;
                }

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

    // Print summary
    let has_issues = errors.has_failures() || !ctx.skipped_phases.is_empty();

    if has_issues {
        print_summary(&errors, &ctx);

        if errors.has_failures() {
            bail!("macup completed with errors");
        } else {
            // Only skipped phases, not a hard error
            println!(
                "\n{}",
                "⚠️  Some phases were skipped due to missing dependencies".yellow()
            );
        }
    }

    println!("{}", "=".repeat(50).bright_green());
    println!("{}", "✓ macup apply completed!".bright_green().bold());
    println!("{}", "=".repeat(50).bright_green());

    Ok(())
}

/// Check if a phase can execute based on satisfied dependencies
fn can_execute_phase(phase: &crate::executor::Phase, available_managers: &HashSet<String>) -> bool {
    // Managers phase can always run
    if matches!(phase.section_type, SectionType::Managers) {
        return true;
    }

    // Package manager phases: Always run, they handle dependencies internally
    // They check actual runtime availability (mas/node/cargo), not brew dependency
    // This allows flexibility: if user has node installed manually, npm phase still works
    if matches!(
        phase.section_type,
        SectionType::Brew | SectionType::Mas | SectionType::Npm | SectionType::Cargo
    ) {
        return true;
    }

    // Install scripts and System commands: Strict dependency checking
    // These truly need their dependencies to work
    for dep in &phase.depends_on {
        if !available_managers.contains(dep) {
            return false;
        }
    }

    true
}

fn check_and_install_manager(name: &str, dry_run: bool) -> Result<()> {
    let exists = crate::utils::command_exists(name);

    if exists {
        println!("  ✓ {} is installed", name.green());
        return Ok(());
    }

    // Not installed
    println!("  → Installing {}...", name.yellow());

    if dry_run {
        println!("    → Would install {}", name);
        return Ok(());
    }

    match name {
        "brew" => {
            let status = Command::new("sh")
                .arg("-c")
                .arg(r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#)
                .status()
                .context("Failed to execute brew install script")?;

            if !status.success() {
                bail!("Homebrew installation failed");
            }

            // Add to PATH for Apple Silicon Macs
            if Path::new("/opt/homebrew/bin/brew").exists() {
                let current_path = std::env::var("PATH").unwrap_or_default();
                std::env::set_var("PATH", format!("/opt/homebrew/bin:{}", current_path));
            }

            println!("  ✓ {} installed", name.green());
        }
        _ => {
            // Other managers (mas, npm, cargo) are auto-installed inline in their sections
            println!("  ℹ️  {} will be auto-installed when needed", name.cyan());
            return Ok(());
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

/// Install a runtime (node, rust, python, etc.) via brew
fn install_runtime_via_brew(formula: &str) -> Result<()> {
    // Check brew exists first
    if !crate::utils::command_exists("brew") {
        bail!("{} requires brew, but brew is not installed", formula);
    }

    let status = Command::new("brew")
        .env("HOMEBREW_NO_AUTO_UPDATE", "1")
        .args(["install", formula])
        .status()
        .context(format!("Failed to execute brew install {}", formula))?;

    if !status.success() {
        bail!("brew install {} failed", formula);
    }

    Ok(())
}

/// Print comprehensive summary at end of apply
fn print_summary(errors: &ApplyErrors, ctx: &ExecutionContext) {
    println!();
    println!("{}", "=".repeat(50).yellow());
    println!("{}", "⚠️  macup completed with issues".yellow().bold());
    println!("{}", "=".repeat(50).yellow());
    println!();

    // Print skipped phases first
    if !ctx.skipped_phases.is_empty() {
        println!("{}", "Skipped phases:".yellow().bold());
        for skipped in &ctx.skipped_phases {
            println!("  ⊘ {} phase", skipped.name.yellow());
            println!("     Reason: {}", skipped.reason);
            println!();
        }
    }

    if !errors.manager_failures.is_empty() {
        println!("{}", "Failed manager installations:".red().bold());
        for failure in &errors.manager_failures {
            println!("  ❌ {} ({})", failure.name.red(), "manager");
            println!("     Reason: {}", failure.reason);
            println!(
                "     Fix: Install {} manually and re-run macup apply",
                failure.name
            );
            println!();
        }
    }

    if !errors.package_failures.is_empty() {
        println!("{}", "Failed package installations:".red().bold());

        // Group by manager for cleaner output
        let mut by_manager: std::collections::HashMap<String, Vec<&PackageFailure>> =
            std::collections::HashMap::new();

        for failure in &errors.package_failures {
            by_manager
                .entry(failure.manager.clone())
                .or_insert_with(Vec::new)
                .push(failure);
        }

        for (manager, failures) in by_manager {
            println!("  {} via {}:", "Packages".red(), manager);
            for failure in failures {
                println!("    ❌ {}", failure.package);
                println!("       Reason: {}", failure.reason);
            }
            println!();
        }
    }

    println!(
        "💡 {}",
        "Run 'macup apply' again after fixing the issues.".bright_yellow()
    );
    println!("   Already installed packages will be skipped automatically.");
    println!();
}
