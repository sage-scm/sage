use anyhow::{Result, anyhow};
use std::process::Command;

/// Stash the current changes
pub fn stash_push(message: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("stash").arg("push");

    if let Some(msg) = message {
        cmd.arg("-m").arg(msg);
    }

    let output = cmd.output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No local changes to save") {
            // Not an error - just no changes to stash
            Ok(())
        } else {
            Err(anyhow!("Failed to stash changes: {}", stderr.trim()))
        }
    }
}

/// Pop the last stash
pub fn stash_pop() -> Result<()> {
    let output = Command::new("git").args(["stash", "pop"]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No stash entries found") {
            // Not an error - just no stash to pop
            Ok(())
        } else if stderr.contains("CONFLICT") {
            Err(anyhow!(
                "Stash pop resulted in conflicts. Please resolve them manually"
            ))
        } else {
            Err(anyhow!("Failed to pop stash: {}", stderr.trim()))
        }
    }
}

/// Check if there are any stashes
pub fn has_stash() -> Result<bool> {
    let output = Command::new("git").args(["stash", "list"]).output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(!stdout.trim().is_empty())
    } else {
        Err(anyhow!("Failed to check stash status"))
    }
}
