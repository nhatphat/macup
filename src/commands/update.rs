use anyhow::{Context, Result};
use colored::Colorize;
use std::process::{Command, Stdio};

const INSTALL_SCRIPT_URL: &str =
    "https://raw.githubusercontent.com/nhatphat/macup/master/install.sh";

pub fn run() -> Result<()> {
    let current_exe = std::env::current_exe().context("Failed to determine current binary path")?;
    let install_dir = current_exe
        .parent()
        .context("Failed to determine current binary directory")?;

    println!(
        "{}",
        "Updating macup to the latest release...".bright_cyan()
    );
    println!("Install directory: {}", install_dir.display());
    println!();

    let status = Command::new("bash")
        .arg("-c")
        .arg(format!("curl -fsSL {} | bash", INSTALL_SCRIPT_URL))
        .env("MACUP_INSTALL_DIR", install_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run macup installer")?;

    if !status.success() {
        anyhow::bail!("macup update failed");
    }

    Ok(())
}
