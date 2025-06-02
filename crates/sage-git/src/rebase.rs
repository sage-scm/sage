use anyhow::{anyhow, Result};
use std::process::Command;

/// Rebase the current branch onto the specified target branch.
///
/// # Arguments
/// * `target` - The branch to rebase onto
/// * `interactive` - Whether to perform an interactive rebase
/// * `autostash` - Whether to automatically stash and unstash changes if needed
///
/// # Returns
/// * `Ok(())` if the rebase was successful
/// * `Err` with an error message if the rebase failed
pub fn rebase(target: &str, interactive: bool, autostash: bool) -> Result<()> {
    let mut cmd = Command::new("git");

    // Start building the rebase command
    cmd.arg("rebase");

    // Add interactive flag if requested
    if interactive {
        cmd.arg("-i");
    }

    // Add autostash flag if requested
    if autostash {
        cmd.arg("--autostash");
    }

    // Add the target branch
    cmd.arg(target);

    // Execute the command
    let output = cmd.output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Provide more helpful error messages for common rebase scenarios
        if stderr.contains("could not apply") {
            Err(anyhow!(
                "Rebase encountered conflicts. Resolve them and run 'git rebase --continue'\n{}",
                stderr
            ))
        } else if stderr.contains("no such branch") {
            Err(anyhow!(
                "Target branch '{}' does not exist. {}",
                target,
                stderr.trim()
            ))
        } else if stderr.contains("not a valid object") {
            Err(anyhow!("Invalid reference '{}'. {}", target, stderr.trim()))
        } else if stderr.contains("is up to date") {
            // This is actually a success case - the branch is already up to date
            Ok(())
        } else if stderr.contains("your local changes would be overwritten") {
            Err(anyhow!(
                "Local changes would be overwritten. Please commit or stash your changes before rebasing.\n{}",
                stderr
            ))
        } else {
            // Generic error for all other cases
            Err(anyhow!("Rebase failed: {}", stderr.trim()))
        }
    }
}
