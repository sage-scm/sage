use anyhow::Result;
use std::process::Command;

/// Check if we're in a git repo.
pub fn in_repo() -> Result<bool> {
    let result = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()?;

    let stdout = String::from_utf8(result.stdout)?;
    Ok(stdout.trim().to_string().eq("true"))
}

pub fn get_repo_root() -> Result<String> {
    let result = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()?;

    let stdout = String::from_utf8(result.stdout)?;
    Ok(stdout.trim().to_string())
}
