use anyhow::{anyhow, Result};
use std::process::Command;

// Commit an empty git commit and return the short commit ID
pub fn commit_empty() -> Result<String> {
    // Create the empty commit
    let output = Command::new("git")
        .args(["commit", "--allow-empty"])
        .output()?;

    if !output.status.success() {
        // Convert stderr to a string for a more detailed error message
        let error_message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!("Failed to create empty commit: {}", error_message));
    }

    // Get the short commit ID of the latest commit
    let id_output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()?;

    if !id_output.status.success() {
        let error_message = String::from_utf8_lossy(&id_output.stderr).trim().to_string();
        return Err(anyhow!("Failed to get commit ID: {}", error_message));
    }

    // Convert the output to a string and trim whitespace
    let commit_id = String::from_utf8(id_output.stdout)?.trim().to_string();

    Ok(commit_id)
}

// Create a commit with the given message and return the short commit ID
pub fn commit(message: &str) -> Result<String> {
    // Create the commit
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()?;

    if !output.status.success() {
        // Convert stderr to a string for a more detailed error message
        let error_message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!("Failed to commit: {}", error_message));
    }

    // Get the short commit ID of the latest commit
    let id_output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()?;

    if !id_output.status.success() {
        let error_message = String::from_utf8_lossy(&id_output.stderr).trim().to_string();
        return Err(anyhow!("Failed to get commit ID: {}", error_message));
    }

    // Convert the output to a string and trim whitespace
    let commit_id = String::from_utf8(id_output.stdout)?.trim().to_string();

    Ok(commit_id)
}
