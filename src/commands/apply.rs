use crate::config::{load_config_auto, validate_config};
use crate::executor::{apply_plan, create_execution_plan};
use anyhow::Result;
use std::path::Path;

pub fn run(config_path: Option<&Path>, dry_run: bool, _section: Option<&str>) -> Result<()> {
    // Load config
    let (path, config) = load_config_auto(config_path)?;

    log::info!("Loaded config from: {}", path.display());

    // Validate config
    validate_config(&config)?;

    // Create execution plan
    let plan = create_execution_plan(&config)?;

    // Apply plan
    apply_plan(&config, &plan, dry_run)?;

    Ok(())
}
