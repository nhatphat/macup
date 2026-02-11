mod cli;
mod commands;
mod config;
mod executor;
mod managers;
mod system;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, NewResource, RemoveResource};

fn main() -> Result<()> {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    // Set verbose logging if requested
    if cli.verbose {
        log::set_max_level(log::LevelFilter::Debug);
    }

    match cli.command {
        Command::Apply {
            dry_run,
            with_system_settings,
            section,
        } => {
            commands::apply::run(
                cli.config.as_deref(),
                dry_run,
                with_system_settings,
                section.as_deref(),
            )?;
        }
        Command::Diff => {
            commands::diff::run(cli.config.as_deref())?;
        }
        Command::Import => {
            commands::import::run(cli.config.as_deref())?;
        }
        Command::Add {
            manager,
            packages,
            no_install,
        } => {
            commands::add::run(cli.config.as_deref(), &manager, packages, no_install)?;
        }
        Command::New { resource } => match resource {
            NewResource::Manager {
                name,
                display,
                icon,
                runtime_cmd,
                runtime_name,
                brew_formula,
            } => {
                commands::new_manager::run(
                    &name,
                    &display,
                    &icon,
                    &runtime_cmd,
                    &runtime_name,
                    &brew_formula,
                )?;
            }
        },
        Command::Remove { resource } => match resource {
            RemoveResource::Manager { name } => {
                commands::remove_manager::run(&name)?;
            }
        },
    }

    Ok(())
}
