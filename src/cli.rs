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
    Diff,

    /// Import packages from current system
    Import,

    /// Add package(s) to config and install
    Add {
        /// Manager type: brew, cask, mas, npm, cargo, gem, pipx, npx
        manager: String,

        /// Package name(s) or ID(s) to add
        packages: Vec<String>,

        /// Only update config, skip installation
        #[arg(long)]
        no_install: bool,
    },

    /// Create a new package manager (developer tool)
    New {
        #[command(subcommand)]
        resource: NewResource,
    },

    /// Remove a package manager (developer tool)
    Remove {
        #[command(subcommand)]
        resource: RemoveResource,
    },
}

#[derive(Subcommand)]
pub enum NewResource {
    /// Generate boilerplate for a new package manager
    Manager {
        /// Manager name (e.g., pip, gem, go)
        name: String,

        /// Display name (e.g., "pip packages")
        #[arg(long)]
        display: String,

        /// Icon emoji (e.g., 🐍)
        #[arg(long)]
        icon: String,

        /// Runtime command to check (e.g., pip3)
        #[arg(long)]
        runtime_cmd: String,

        /// Human-readable runtime name (e.g., python)
        #[arg(long)]
        runtime_name: String,

        /// Brew formula name (e.g., python)
        #[arg(long)]
        brew_formula: String,
    },
}

#[derive(Subcommand)]
pub enum RemoveResource {
    /// Remove a package manager
    Manager {
        /// Manager name (e.g., pip, gem, go)
        name: String,
    },
}
