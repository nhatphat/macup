use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "macup")]
#[command(author, version, about, long_about = None)]
#[command(about = "A thin orchestrator for Mac bootstrap and setup")]
pub struct Cli {
    /// Path to config file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Apply configuration (install packages, apply settings)
    Apply {
        /// Only show what would be done, don't make changes
        #[arg(long)]
        dry_run: bool,

        /// Include system settings (defaults commands)
        #[arg(long)]
        with_system_settings: bool,

        /// Apply only specific section (brew, mas, npm, cargo, install, system)
        section: Option<String>,
    },

    /// Show difference between config and current state
    Diff {
        /// Include system settings check
        #[arg(long)]
        with_system: bool,
    },

    /// Import packages from current system
    Import,

    /// Update macup to the latest release
    Update,

    /// Add package(s) to config and install
    #[command(alias = "a")]
    Add {
        /// Manager type: brew, cask, mas, npm, cargo, gem, pipx, npx
        manager: String,

        /// Package name(s) or ID(s) to add
        packages: Vec<String>,

        /// Only update config, skip installation
        #[arg(long)]
        no_install: bool,
    },

    /// Remove package(s) from config
    #[command(alias = "rm")]
    Remove {
        /// Manager type: brew, cask, npm, cargo, gem, pipx, npx
        manager: String,

        /// Package name(s) to remove
        packages: Vec<String>,
    },

    /// Developer tools for maintaining macup itself
    Dev {
        #[command(subcommand)]
        command: DevCommand,
    },
}

#[derive(Subcommand)]
pub enum DevCommand {
    /// Generate developer boilerplate
    Generate {
        #[command(subcommand)]
        resource: GenerateResource,
    },

    /// Remove developer boilerplate
    Remove {
        #[command(subcommand)]
        resource: DevRemoveResource,
    },
}

#[derive(Subcommand)]
pub enum GenerateResource {
    /// Generate boilerplate for a new package manager
    Manager(ManagerGeneratorArgs),
}

#[derive(Subcommand)]
pub enum DevRemoveResource {
    /// Remove a package manager
    Manager {
        /// Manager name (e.g., pip, gem, go)
        name: String,
    },
}

#[derive(clap::Args)]
pub struct ManagerGeneratorArgs {
    /// Manager name (e.g., pip, gem, go)
    pub name: String,

    /// Display name (e.g., "pip packages")
    #[arg(long)]
    pub display: String,

    /// Icon emoji (e.g., 🐍)
    #[arg(long)]
    pub icon: String,

    /// Runtime command to check (e.g., pip3)
    #[arg(long)]
    pub runtime_cmd: String,

    /// Human-readable runtime name (e.g., python)
    #[arg(long)]
    pub runtime_name: String,

    /// Brew formula name (e.g., python)
    #[arg(long)]
    pub brew_formula: String,
}
