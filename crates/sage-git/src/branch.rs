use anyhow::{Result, anyhow, bail};
use once_cell::sync::Lazy;
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
    let branches: Vec<&str> = stdout
        .lines()
        .map(|line| line.trim().trim_start_matches('*').trim())
        .collect();
    Ok(branches.contains(&branch))
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

/// Cache for the default branch name
static DEFAULT_BRANCH: Lazy<Result<String>> = Lazy::new(|| {
    // Try to get the default branch from the remote
    let result = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            // Format is refs/remotes/origin/main or refs/remotes/origin/master
            let stdout = String::from_utf8_lossy(&output.stdout);
            let branch = stdout
                .trim()
                .strip_prefix("refs/remotes/origin/")
                .unwrap_or("main")
                .to_string();
            Ok(branch)
        }
        _ => {
            // Fallback: try to determine from common default branch names
            for branch in ["main", "master", "develop"] {
                let check = Command::new("git")
                    .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch)])
                    .output();

                if let Ok(output) = check {
                    if output.status.success() {
                        return Ok(branch.to_string());
                    }
                }
            }

            // Last resort: return the current branch
            let result = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .output()?;

            let stdout = String::from_utf8(result.stdout)?;
            Ok(stdout.trim().to_string())
        }
    }
});

/// Get the default branch name.
pub fn get_default_branch() -> Result<String> {
    match &*DEFAULT_BRANCH {
        Ok(branch) => Ok(branch.clone()),
        Err(e) => Err(anyhow!("Failed to determine default branch: {}", e)),
    }
}

/// Check if a given branch is the default branch.
pub fn is_default(branch: &str) -> Result<bool> {
    let head_branch = get_default_branch()?;
    Ok(head_branch == branch)
}

/// Determine if the branch has any changes on it.
pub fn is_clean() -> Result<bool> {
    let result = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()?;

    let stdout = String::from_utf8(result.stdout)?;
    Ok(stdout.trim().is_empty())
}

/// Unstage all changes in the repository
pub fn unstage_all() -> Result<()> {
    let result = Command::new("git").args(["reset", "HEAD"]).output()?;

    if !result.status.success() {
        return Err(anyhow!(
            "Failed to unstage all changes: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    Ok(())
}

/// Stage all changes in the repository
pub fn stage_all() -> Result<()> {
    let result = Command::new("git").args(["add", "--all"]).output()?;

    if !result.status.success() {
        return Err(anyhow!(
            "Failed to stage all changes: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    Ok(())
}

/// List all local branches
pub fn list_branches() -> Result<Vec<String>> {
    let result = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()?;

    if !result.status.success() {
        return Err(anyhow!(
            "Failed to list branches: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    let stdout = String::from_utf8(result.stdout)?;
    let branches = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(branches)
}

/// Pull the current branch from the remote.
pub fn pull() -> Result<()> {
    let current = get_current()?;

    let res = Command::new("git")
        .arg("pull")
        .arg("origin")
        .arg(current)
        .arg("--ff-only")
        .output()?;

    if !res.status.success() {
        bail!(
            "Failed to pull branch: {}",
            String::from_utf8_lossy(&res.stderr)
        )
    }

    Ok(())
}

/// Merge a specific branch into the current branch.
pub fn merge(branch: &str) -> Result<()> {
    let res = Command::new("git").arg("merge").arg(branch).output()?;

    if !res.status.success() {
        let stderr = String::from_utf8_lossy(&res.stderr);
        if stderr.contains("Automatic merge failed") || stderr.contains("CONFLICT") {
            bail!("Merge conflicts detected. Please resolve conflicts and commit the changes");
        }
        bail!("Failed to merge branch: {}", stderr)
    }

    Ok(())
}

/// Check if a merge is in progress
pub fn is_merge_in_progress() -> Result<bool> {
    use std::path::Path;
    let git_dir = Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()?;
    
    if !git_dir.status.success() {
        return Ok(false);
    }
    
    let git_dir_path = String::from_utf8_lossy(&git_dir.stdout).trim().to_string();
    let merge_head = Path::new(&git_dir_path).join("MERGE_HEAD");
    
    Ok(merge_head.exists())
}

/// Abort an in-progress merge
pub fn merge_abort() -> Result<()> {
    let output = Command::new("git")
        .args(&["merge", "--abort"])
        .output()?;
    
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to abort merge: {}", stderr.trim())
    }
}

/// Check if the branch has diverged from its upstream
pub fn has_diverged(branch: &str) -> Result<bool> {
    use crate::status::branch_status;
    
    let status = branch_status(branch)?;
    
    // Has diverged if we have both local commits (ahead) and remote commits (behind)
    Ok(status.ahead_count > 0 && status.behind_count > 0)
}

/// Check if branch is shared (pushed to remote and potentially used by others)
pub fn is_shared_branch(branch: &str) -> Result<bool> {
    use crate::status::branch_status;
    
    let status = branch_status(branch)?;
    
    // Branch is shared if it has an upstream branch
    // This means it exists on the remote and could be used by others
    Ok(status.upstream_branch.is_some())
}

/// Simple ahead and behind for when you dont need the full status
pub fn ahead_behind(base: &str, compare: &str) -> Result<(i32, i32)> {
    let res = Command::new("git")
        .arg("rev-list")
        .arg("--left-right")
        .arg("--count")
        .arg(format!("{}...{}", base, compare))
        .output()?;

    if !res.status.success() {
        bail!(
            "Failed to get ahead and behind for '{}': {}",
            compare,
            String::from_utf8_lossy(&res.stderr)
        )
    }

    let stdout = String::from_utf8(res.stdout)?;
    let parts: Vec<&str> = stdout.trim().split('\t').collect();

    if parts.len() != 2 {
        bail!("Unexpected output format from git rev-list");
    }

    let ahead = parts[0].parse::<i32>()?;
    let behind = parts[1].parse::<i32>()?;

    Ok((ahead, behind))
}
