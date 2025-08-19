use anyhow::bail;
use chrono::{DateTime, Utc};

use crate::prelude::{Git, GitResult};

pub struct CommitResult {
    pub commit_id: String,
    pub hook_output: Option<String>,
}

pub fn commit_empty_with_output() -> GitResult<CommitResult> {
    let output = Git::new("commit").arg("--allow-empty").raw_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create empty commit: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let hook_output = if !stdout.trim().is_empty() {
        Some(stdout.trim().to_string())
    } else {
        None
    };

    let commit_id = last_commit_id()?;
    Ok(CommitResult {
        commit_id,
        hook_output,
    })
}

pub fn commit_empty() -> GitResult<String> {
    let result = commit_empty_with_output()?;
    Ok(result.commit_id)
}

pub fn commit_with_output(message: &str) -> GitResult<CommitResult> {
    let output = Git::new("commit").args(["-m", message]).raw_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create commit: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let hook_output = if !stdout.trim().is_empty() {
        Some(stdout.trim().to_string())
    } else {
        None
    };

    let commit_id = last_commit_id()?;
    Ok(CommitResult {
        commit_id,
        hook_output,
    })
}

pub fn commit(message: &str) -> GitResult<String> {
    let result = commit_with_output(message)?;
    Ok(result.commit_id)
}

pub fn commit_with_file(message: &str, file: &str) -> GitResult<String> {
    Git::new("commit")
        .args(["-m", message, file])
        .context("Failed to create commit with file")
        .run()?;

    last_commit_id()
}

pub fn last_commit_id() -> GitResult<String> {
    Git::new("rev-parse")
        .args(["--short", "HEAD"])
        .context("Failed to get last commit ID")
        .output()
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    pub date: String,
    pub author: String,
}

pub fn commits(limit: usize) -> GitResult<Vec<Commit>> {
    let log_result = if limit == 0 {
        log("", 0, false, true)?
    } else {
        log("", limit, false, false)?
    };
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

pub fn log(branch: &str, limit: usize, stats: bool, all: bool) -> GitResult<Vec<String>> {
    let mut git = Git::new("log").arg("--pretty=format:%H%x00%an%x00%at%x00%s");

    if limit > 0 && !all {
        git = git.arg(&format!("-n {limit}"));
    }

    if stats {
        git = git.arg("--numstat");
    }

    if !branch.is_empty() {
        git = git.arg(branch);
    }

    let output = git.context("Failed to list commits").output()?;

    let commits: Vec<String> = output.split('\n').map(|s| s.to_string()).collect();
    Ok(commits)
}
