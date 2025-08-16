use anyhow::{Result, bail};
use std::process::Command;

use crate::prelude::{git_ok, git_output};

/// Check if we're in a git repo.
pub fn in_repo() -> Result<bool> {
    let result = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()?;

    let stdout = String::from_utf8(result.stdout)?;
    Ok(stdout.trim().to_string().eq("true"))
}

/// Gets the root directory of the repo.
pub fn get_repo_root() -> Result<String> {
    git_output(["rev-parse", "--show-toplevel"])
}

/// Fetches the latest from remote.
pub fn fetch_remote() -> Result<()> {
    git_ok(["fetch", "--all", "--prune"])
}

/// Get the diff between the current branch and the remote branch with enhanced context.
/// Provides rich context for AI commit message generation.
pub fn diff() -> Result<String> {
    use crate::status::{StatusType, get_status_entries};

    // Get all status entries to understand what files are changed
    let status_entries = get_status_entries()?;

    // Prepare a summary of changes for better context
    let mut summary = String::new();

    // Group files by status type
    let staged_files: Vec<_> = status_entries
        .iter()
        .filter(|e| e.status_type == StatusType::Staged)
        .collect();

    let unstaged_files: Vec<_> = status_entries
        .iter()
        .filter(|e| e.status_type == StatusType::Unstaged)
        .collect();

    let untracked_files: Vec<_> = status_entries
        .iter()
        .filter(|e| e.status_type == StatusType::Untracked)
        .collect();

    // Add file summary to the context
    summary.push_str("# Files Changed\n\n");

    if !staged_files.is_empty() {
        summary.push_str("## Staged Files:\n");
        for entry in &staged_files {
            summary.push_str(&format!("- {} ({})\n", entry.path, entry.status_code));
        }
        summary.push('\n');
    }

    if !unstaged_files.is_empty() {
        summary.push_str("## Unstaged Files:\n");
        for entry in &unstaged_files {
            summary.push_str(&format!("- {} ({})\n", entry.path, entry.status_code));
        }
        summary.push('\n');
    }

    if !untracked_files.is_empty() {
        summary.push_str("## Untracked Files:\n");
        for entry in &untracked_files {
            summary.push_str(&format!("- {}\n", entry.path));
        }
        summary.push('\n');
    }

    // Get the actual diff content
    let has_staged = !staged_files.is_empty();
    let mut diff_content = String::new();

    if has_staged {
        // Get staged changes with optimized parameters
        let output = Command::new("git")
            .args(["diff", "--cached", "--patch"])
            .output()?;

        diff_content = String::from_utf8(output.stdout)?;
    } else if !unstaged_files.is_empty() {
        // No staged changes, get unstaged changes
        let output = Command::new("git").args(["diff", "--patch"]).output()?;

        diff_content = String::from_utf8(output.stdout)?;
    }

    // For untracked files, get their content if they're small text files
    if !untracked_files.is_empty() && diff_content.is_empty() {
        for entry in &untracked_files {
            // Only process small text files (avoid binary files)
            let file_check = Command::new("file")
                .args(["-b", "--mime-type", &entry.path])
                .output();

            if let Ok(check) = file_check {
                let mime = String::from_utf8_lossy(&check.stdout);
                if mime.starts_with("text/") {
                    // It's a text file, get its content
                    let cat = Command::new("cat").arg(&entry.path).output();

                    if let Ok(content) = cat {
                        let file_content = String::from_utf8_lossy(&content.stdout);
                        if file_content.len() < 1000 {
                            // Only include small files
                            diff_content
                                .push_str(&format!("\n--- /dev/null\n+++ b/{}\n", entry.path));
                            for line in file_content.lines() {
                                diff_content.push_str(&format!("+{}\n", line));
                            }
                        }
                    }
                }
            }
        }
    }

    // Combine summary and diff content
    Ok(format!("{}\n# Diff Content\n\n{}", summary, diff_content))
}

/// Get commiter details
pub fn get_commiter() -> Result<(String, String)> {
    let name = git_output(["config", "user.name"])?;
    let email = git_output(["config", "user.email"])?;
    Ok((name, email))
}

/// get the owner and repo name from the remote URL
pub fn owner_repo() -> Result<(String, String)> {
    let name = name().unwrap();
    if let Some(owner_repo) = name.split('/').next() {
        let (owner, repo) = owner_repo.split_once('/').unwrap();
        Ok((owner.to_string(), repo.to_string()))
    } else {
        bail!("Could not parse owner and repo from remote URL");
    }
}

/// Check if there are any unresolved conflicts in the repository
pub fn has_conflicts() -> Result<bool> {
    let output = git_output(["diff", "--name-only", "--diff-filter=U"])?;
    Ok(!output.trim().is_empty())
}

/// Get repo name
pub fn name() -> Option<String> {
    if let Ok(url) = git_output(["config", "--get", "remote.origin.url"]) {
        let name = url
            .trim_end_matches(".git")
            .rsplit('/')
            .next()
            .or_else(|| url.rsplit(':').next())
            .unwrap_or("")
            .to_string();

        if !name.is_empty() {
            return Some(name);
        }
    }

    // fallback: current dir
    std::env::current_dir()
        .ok()?
        .file_name()?
        .to_str()
        .map(|s| s.to_string())
}
