use anyhow::Result;
use std::path::Path;

pub fn run(config_path: Option<&Path>) -> Result<()> {
    crate::import::run(config_path)
}
