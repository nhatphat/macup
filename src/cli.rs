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

    /// Add package(s) to config and install
    Add {
        /// Manager type: brew, cask, mas, npm, cargo
        manager: String,

        /// Package name(s) or ID(s) to add
        packages: Vec<String>,

        /// Only update config, skip installation
        #[arg(long)]
        no_install: bool,
    },
}
