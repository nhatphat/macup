use crate::cli::{Cli, Command, DevCommand, DevRemoveResource, GenerateResource};
use crate::commands;
use anyhow::Result;

pub fn run(cli: Cli) -> Result<()> {
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
        Command::Diff { with_system } => {
            commands::diff::run(cli.config.as_deref(), with_system)?;
        }
        Command::Import => {
            commands::import::run(cli.config.as_deref())?;
        }
        Command::Update => {
            commands::update::run()?;
        }
        Command::Add {
            manager,
            packages,
            no_install,
        } => {
            commands::add::run(cli.config.as_deref(), &manager, packages, no_install)?;
        }
        Command::Remove { manager, packages } => {
            commands::remove::run(cli.config.as_deref(), &manager, packages)?;
        }
        Command::Dev { command } => match command {
            DevCommand::Generate { resource } => match resource {
                GenerateResource::Manager(args) => {
                    commands::new_manager::run(
                        &args.name,
                        &args.display,
                        &args.icon,
                        &args.runtime_cmd,
                        &args.runtime_name,
                        &args.brew_formula,
                    )?;
                }
            },
            DevCommand::Remove { resource } => match resource {
                DevRemoveResource::Manager { name } => {
                    commands::remove_manager::run(&name)?;
                }
            },
        },
    }

    Ok(())
}
