use anyhow::Result;
use std::path::Path;

pub fn run(config_path: Option<&Path>, with_system: bool) -> Result<()> {
    crate::diff::run(config_path, with_system)
}
