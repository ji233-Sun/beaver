use anyhow::{Context, Result};
use std::process::{Command, ExitStatus};

/// Execute a shell command string via `sh -c` and return its exit status.
pub fn run(command: &str) -> Result<ExitStatus> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .with_context(|| format!("Failed to execute: {}", command))?;
    Ok(status)
}

/// Execute a command and propagate its exit code to the process.
pub fn run_and_exit(command: &str) -> Result<()> {
    let status = run(command)?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
