use anyhow::bail;
use std::process::Command;

use crate::prelude::{Git, GitResult};

pub fn in_repo() -> GitResult<bool> {
    let output = Git::new("rev-parse")
        .arg("--is-inside-work-tree")
        .raw_output()?;

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.trim() == "true")
}

pub fn get_repo_root() -> GitResult<String> {
    Git::new("rev-parse")
        .arg("--show-toplevel")
        .context("Failed to get repository root")
        .output()
}

pub fn fetch_remote() -> GitResult<()> {
    Git::new("fetch")
        .args(["--all", "--prune"])
        .context("Failed to fetch from remote")
        .run()
}

pub fn diff() -> GitResult<String> {
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
        let output = Git::new("diff")
            .args(["--cached", "--patch"])
            .raw_output()?;

        diff_content = String::from_utf8(output.stdout)?;
    } else if !unstaged_files.is_empty() {
        let output = Git::new("diff").arg("--patch").raw_output()?;

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
                                diff_content.push_str(&format!("+{line}\n"));
                            }
                        }
                    }
                }
            }
        }
    }

    // Combine summary and diff content
    Ok(format!("{summary}\n# Diff Content\n\n{diff_content}"))
}

pub fn get_commiter() -> GitResult<(String, String)> {
    let name = Git::new("config").arg("user.name").output()?;
    let email = Git::new("config").arg("user.email").output()?;
    Ok((name, email))
}

pub fn owner_repo() -> GitResult<(String, String)> {
    let name = name().unwrap();
    if let Some(owner_repo) = name.split('/').next() {
        let (owner, repo) = owner_repo.split_once('/').unwrap();
        Ok((owner.to_string(), repo.to_string()))
    } else {
        bail!("Could not parse owner and repo from remote URL");
    }
}

pub fn has_conflicts() -> GitResult<bool> {
    let output = Git::new("diff")
        .args(["--name-only", "--diff-filter=U"])
        .output()?;
    Ok(!output.trim().is_empty())
}

pub fn name() -> Option<String> {
    if let Ok(url) = Git::new("config")
        .args(["--get", "remote.origin.url"])
        .output()
    {
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
