use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
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
        let error_message = String::from_utf8_lossy(&id_output.stderr)
            .trim()
            .to_string();
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
        let error_message = String::from_utf8_lossy(&id_output.stderr)
            .trim()
            .to_string();
        return Err(anyhow!("Failed to get commit ID: {}", error_message));
    }

    // Convert the output to a string and trim whitespace
    let commit_id = String::from_utf8(id_output.stdout)?.trim().to_string();

    Ok(commit_id)
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    pub date: String,
    pub author: String,
}

// Get the commits for the current branch
pub fn commits() -> Result<Vec<Commit>> {
    let log_result = log("", 0, false, true)?;
    let mut commits = Vec::new();

    for log_line in log_result {
        let parts: Vec<&str> = log_line.split('\x00').collect();
        if parts.len() < 4 {
            continue; // We want to avoid these lines, as they are not proper commits.
        }

        let hash = parts[0].to_string();
        let author = parts[1];
        let timestamp = parts[2];
        let message = parts[3];

        // Format the date from Unix timestamp
        let formatted_date = if let Ok(ts) = timestamp.parse::<i64>() {
            // Use the non-deprecated approach
            if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
                format!("{}", dt.format("%a %b %d %Y"))
            } else {
                "Unknown date".to_string()
            }
        } else {
            "Unknown date".to_string()
        };

        // Get short hash (first 7 characters)
        let short_hash = if hash.len() >= 7 {
            hash[..7].to_string()
        } else {
            hash.clone()
        };

        commits.push(Commit {
            hash: short_hash,
            author: author.to_string(),
            date: formatted_date,
            message: message.to_string(),
        });
    }

    Ok(commits)
}

pub fn log(branch: &str, limit: usize, stats: bool, all: bool) -> Result<Vec<String>> {
    let mut cmd = Command::new("git");
    cmd.arg("log");
    cmd.arg("--pretty=format:%H%x00%an%x00%at%x00%s");

    if limit > 0 && !all {
        cmd.arg(format!("-n {}", limit));
    }

    if stats {
        cmd.arg("--numstat");
    }

    if !branch.is_empty() {
        cmd.arg(branch);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to list commits"));
    }

    let output = String::from_utf8(output.stdout)?;
    let commits: Vec<String> = output.split('\n').map(|s| s.to_string()).collect();
    Ok(commits)
}
