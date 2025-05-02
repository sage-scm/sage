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

/// Switch to a branch, and optionally create it if it doesn't exist.
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

/// Sets the upstream branch for the current branch.
pub fn set_upstream(name: &str) -> Result<()> {
    let result = Command::new("git")
        .arg("branch")
        .arg("--set-upstream-to")
        .arg(format!("origin/{name}"))
        .output()?;

    if !result.status.success() {
        return Err(anyhow!(
            "Failed to set upstream branch: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    Ok(())
}

/// Pushes the current branch to the remote.
pub fn push(name: &str, force: bool) -> Result<()> {
    // Create a git push command
    let mut cmd = Command::new("git");
    cmd.arg("push")
        .arg("--set-upstream")
        .arg("origin")
        .arg(name);

    // Add force options based on the force flag
    if force {
        cmd.arg("--force");
    } else {
        cmd.arg("--force-with-lease");
    }

    // Execute the command
    let result = cmd.output()?;

    if result.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "Failed to push branch: {}",
            String::from_utf8_lossy(&result.stderr)
        ))
    }
}

/// Check if the current branch is the default branch (main, master).
pub fn is_default_branch() -> Result<bool> {
    let head_branch = get_default_branch()?;
    let current = get_current()?;
    Ok(head_branch == current)
}

/// Get the default branch name.
pub fn get_default_branch() -> Result<String> {
    let result = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()?;

    let stdout = String::from_utf8(result.stdout)?;
    Ok(stdout.trim().to_string())
}

/// Check if a given branch is the default branch.
pub fn is_default(branch: &str) -> Result<bool> {
    let head_branch = get_default_branch()?;
    Ok(head_branch == branch)
}
