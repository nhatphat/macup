use super::state::{ApplyErrors, ExecutionContext, PackageFailure};
use colored::Colorize;
use std::collections::HashMap;

/// Print comprehensive summary at end of apply.
pub(super) fn print_summary(errors: &ApplyErrors, ctx: &ExecutionContext) {
    println!();
    println!("{}", "=".repeat(50).yellow());
    println!("{}", "⚠️  macup completed with issues".yellow().bold());
    println!("{}", "=".repeat(50).yellow());
    println!();

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
            println!("  ❌ {} (manager)", failure.name.red());
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

        let mut by_manager: HashMap<String, Vec<&PackageFailure>> = HashMap::new();
        for failure in &errors.package_failures {
            by_manager
                .entry(failure.manager.clone())
                .or_default()
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
