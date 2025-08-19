use crate::prelude::{Git, GitResult};
use anyhow::anyhow;

pub fn is_rebase_in_progress() -> GitResult<bool> {
    use std::path::Path;

    let output = Git::new("rev-parse").arg("--git-dir").raw_output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to find git directory"));
    }

    let git_dir_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let rebase_merge = Path::new(&git_dir_path).join("rebase-merge");
    let rebase_apply = Path::new(&git_dir_path).join("rebase-apply");

    Ok(rebase_merge.exists() || rebase_apply.exists())
}

pub fn rebase_continue() -> GitResult<()> {
    let output = Git::new("rebase").arg("--continue").raw_output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to continue rebase: {}", stderr.trim()))
    }
}

pub fn rebase_abort() -> GitResult<()> {
    let output = Git::new("rebase").arg("--abort").raw_output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to abort rebase: {}", stderr.trim()))
    }
}

pub fn rebase(target: &str, interactive: bool, autostash: bool) -> GitResult<()> {
    let mut git = Git::new("rebase");

    if interactive {
        git = git.arg("-i");
    }

    if autostash {
        git = git.arg("--autostash");
    }

    git = git.arg(target);

    let output = git.raw_output()?;

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
            Ok(())
        } else if stderr.contains("your local changes would be overwritten") {
            Err(anyhow!(
                "Local changes would be overwritten. Please commit or stash your changes before rebasing.\n{}",
                stderr
            ))
        } else {
            Err(anyhow!("Rebase failed: {}", stderr.trim()))
        }
    }
}
