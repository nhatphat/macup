use anyhow::Result;
use std::process::{Command, Output};

/// Execute a command and return output
pub fn execute_command(program: &str, args: &[&str]) -> Result<Output> {
    log::debug!("Executing: {} {}", program, args.join(" "));

    let output = Command::new(program).args(args).output()?;

    Ok(output)
}

/// Execute a command and check if it succeeds
pub fn execute_command_success(program: &str, args: &[&str]) -> Result<bool> {
    let output = execute_command(program, args)?;
    Ok(output.status.success())
}

/// Execute a shell command
pub fn execute_shell(command: &str) -> Result<Output> {
    log::debug!("Executing shell: {}", command);

    let output = Command::new("sh").arg("-c").arg(command).output()?;

    Ok(output)
}

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}
