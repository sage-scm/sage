use anyhow::{Result, anyhow, bail};
use once_cell::sync::Lazy;
use std::process::Command;

use crate::prelude::{git_ok, git_output, git_success};

/// Get the current branch name.
pub fn get_current() -> Result<String> {
    git_output(["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Check if the branch exists locally.
pub fn exists(branch: &str) -> Result<bool> {
    let output = git_output(["branch", "--list", branch])?;
    let branches: Vec<&str> = output
        .lines()
        .map(|line| line.trim().trim_start_matches('*').trim())
        .collect();

    Ok(branches.contains(&branch))
}

/// Switch to a branch, and optionally create it if it doesn't exist.
pub fn switch(name: &str, create: bool) -> Result<String> {
    let current_branch = get_current()?;
    let mut args = vec!["switch"];

    if create {
        args.push("-c")
    }
    args.push(name);

    git_ok(&args)?;

    Ok(current_branch)
}

/// Sets the upstream branch for the current branch.
pub fn set_upstream(name: &str) -> Result<()> {
    git_ok(["branch", "--set-upstream-to", &format!("origin/{name}")])
}

/// Pushes the current branch to the remote.
pub fn push(name: &str, force: bool) -> Result<()> {
    let mut args = vec!["push", "--set-upstream", "origin", name];

    // Add force options based on the force flag
    if force {
        args.push("--force");
    } else {
        args.push("--force-with-lease");
    }

    git_ok(&args)
}

/// Check if the current branch is the default branch (main, master).
pub fn is_default_branch() -> Result<bool> {
    let head_branch = get_default_branch()?;
    let current = get_current()?;
    Ok(head_branch == current)
}

/// Cache for the default branch name
static DEFAULT_BRANCH: Lazy<Result<String>> = Lazy::new(|| {
    if let Ok(sym) = git_output(["symbolic-ref", "refs/remotes/origin/HEAD"]) {
        if let Some(tail) = sym.rsplit('/').next() {
            return Ok(tail.to_string());
        }
    }
    // Fallback to `main` then `master`, then the current branch
    if git_success(["show-ref", "--verify", "refs/heads/main"]).unwrap_or(false) {
        return Ok("main".into());
    }
    if git_success(["show-ref", "--verify", "refs/heads/master"]).unwrap_or(false) {
        return Ok("master".into());
    }
    get_current()
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
    git_output(["status", "--porcelain"]).map(|stdout| stdout.trim().is_empty())
}

/// Unstage all changes in the repository
pub fn unstage_all() -> Result<()> {
    git_ok(["restore", "--staged", "."])
}

/// Stage all changes in the repository
pub fn stage_all() -> Result<()> {
    git_ok(["add", "--all"])
}

/// List all local branches
#[derive(Debug, Default)]
pub struct BranchList {
    pub branches: Vec<String>,
    pub local: Vec<String>,
    pub remote: Vec<String>,
}
pub fn list_branches() -> Result<BranchList> {
    let output = git_output(["branch", "-a", "--format=%(refname:short)"])?;
    let mut branches = BranchList::default();
    for line in output.lines() {
        // Skip the origin and HEAD definitions
        if line.trim() == "origin" || line.trim() == "origin/HEAD" || line.trim() == "HEAD" {
            continue;
        }
        branches.branches.push(line.trim().to_string());

        if line.starts_with("origin/") {
            branches
                .remote
                .push(line.trim().strip_prefix("origin/").unwrap().to_string());
        } else {
            branches.local.push(line.trim().to_string());
        }
    }

    Ok(branches)
}

/// Pull the current branch from the remote.
pub fn pull() -> Result<()> {
    let current = get_current()?;
    git_ok(["pull", "origin", &current, "--ff-only"])
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
    git_ok(["merge", "--abort"])
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

/// Check if there is a remote branch with the specified name.
pub fn has_remote_branch(branch: &str) -> Result<bool> {
    let remotes = git_output(["remote"])?;
    Ok(remotes.lines().any(|r| r.trim() == branch))
}

/// Check if the current branch is tracking a remote branch.
pub fn current_upstream() -> Result<String> {
    git_output(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
}
