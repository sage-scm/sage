use anyhow::{anyhow, Result};
use std::process::Command;

/// Get the current branch name.
pub fn get_current() -> Result<String> {
    let result = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()?;

    let stdout = String::from_utf8(result.stdout)?;
    Ok(stdout.trim().to_string())
}

/// Check if the branch exists locally.
pub fn exists(branch: &str) -> Result<bool> {
    let result = Command::new("git")
        .arg("branch")
        .arg("--list")
        .arg(branch)
        .output()?;

    let stdout = String::from_utf8(result.stdout)?;
    Ok(stdout.trim().to_string().eq(branch))
}

pub fn switch(name: &str, create: bool) -> Result<String> {
    let current_branch = get_current()?;
    let mut cmd = Command::new("git");

    cmd.arg("switch");
    if create {
        cmd.arg("-c");
    }
    cmd.arg(name);

    let output = cmd.output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to switch branch: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(current_branch)
}
