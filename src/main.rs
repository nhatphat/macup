use anyhow::Result;
use clap::Parser;
use macup::cli::{dispatch, Cli};

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

    dispatch::run(cli)
}
