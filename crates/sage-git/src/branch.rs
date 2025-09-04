use anyhow::{anyhow, bail};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs;
use std::process::Command;

use crate::prelude::{Git, GitResult, parse_branch_lines};

/// Get the current branch name.
pub fn get_current() -> GitResult<String> {
    Git::new("rev-parse")
        .args(["--abbrev-ref", "HEAD"])
        .context("Failed to get current branch")
        .output()
}

/// Check if the branch exists locally.
pub fn exists(branch: &str) -> GitResult<bool> {
    let output = Git::new("branch")
        .args(["--list", branch])
        .context("Failed to list branches")
        .output()?;

    let branches = parse_branch_lines(&output);
    Ok(branches.contains(&branch.to_string()))
}

pub fn switch(name: &str, create: bool) -> GitResult<String> {
    let current_branch = get_current()?;

    let mut git = Git::new("switch");
    if create {
        git = git.arg("-c");
    }
    git.arg(name).context("Failed to switch branch").run()?;

    Ok(current_branch)
}

pub fn set_upstream(name: &str) -> GitResult<()> {
    Git::new("branch")
        .args(["--set-upstream-to", &format!("origin/{name}")])
        .context("Failed to set upstream branch")
        .run()
}

pub fn push(name: &str, force: bool) -> GitResult<()> {
    let mut git = Git::new("push").args(["--set-upstream", "origin", name]);

    if force {
        git = git.arg("--force");
    } else {
        git = git.arg("--force-with-lease");
    }

    git.context("Failed to push branch").run()
}

pub fn is_default_branch() -> GitResult<bool> {
    let head_branch = get_default_branch()?;
    let current = get_current()?;
    Ok(head_branch == current)
}

static DEFAULT_BRANCH: Lazy<GitResult<String>> = Lazy::new(|| {
    if let Ok(sym) = Git::new("symbolic-ref")
        .args(["refs/remotes/origin/HEAD"])
        .output()
        && let Some(tail) = sym.rsplit('/').next()
    {
        return Ok(tail.to_string());
    }

    if Git::new("show-ref")
        .args(["--verify", "refs/heads/main"])
        .success()
        .unwrap_or(false)
    {
        return Ok("main".into());
    }

    if Git::new("show-ref")
        .args(["--verify", "refs/heads/master"])
        .success()
        .unwrap_or(false)
    {
        return Ok("master".into());
    }

    get_current()
});

pub fn get_default_branch() -> GitResult<String> {
    match &*DEFAULT_BRANCH {
        Ok(branch) => Ok(branch.clone()),
        Err(e) => Err(anyhow!("Failed to determine default branch: {}", e)),
    }
}

pub fn is_default(branch: &str) -> GitResult<bool> {
    let head_branch = get_default_branch()?;
    Ok(head_branch == branch)
}

pub fn is_clean() -> GitResult<bool> {
    Git::new("status")
        .arg("--porcelain")
        .output()
        .map(|stdout| stdout.trim().is_empty())
}

pub fn unstage_all() -> GitResult<()> {
    Git::new("restore")
        .args(["--staged", "."])
        .context("Failed to unstage changes")
        .run()
}

pub fn stage_all() -> GitResult<()> {
    Git::new("add")
        .arg("--all")
        .context("Failed to stage changes")
        .run()
}

/// Stage a specific set of file paths.
///
/// Accepts any iterable of items that can be referenced as `&str`,
/// for example `Vec<String>` or `Vec<&str>`.
pub fn stage_paths<I, S>(paths: I) -> GitResult<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let paths_vec: Vec<String> = paths.into_iter().map(|p| p.as_ref().to_string()).collect();

    if paths_vec.is_empty() {
        // Nothing to stage; treat as success.
        return Ok(());
    }

    Git::new("add")
        .arg("--")
        .args(&paths_vec)
        .context("Failed to stage provided paths")
        .run()
}

/// List all local branches
#[derive(Debug, Default)]
pub struct BranchList {
    pub branches: Vec<String>,
    pub local: Vec<String>,
    pub remote: Vec<String>,
}
pub fn list_branches() -> GitResult<BranchList> {
    let output = Git::new("branch")
        .args(["-a", "--format=%(refname:short)"])
        .context("Failed to list branches")
        .output()?;
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

pub fn pull() -> GitResult<()> {
    let current = get_current()?;
    Git::new("pull")
        .args(["origin", &current, "--ff-only"])
        .context("Failed to pull branch")
        .run()
}

pub fn merge(branch: &str) -> GitResult<()> {
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

pub fn is_merge_in_progress() -> GitResult<bool> {
    use std::path::Path;
    let git_dir = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()?;

    if !git_dir.status.success() {
        return Ok(false);
    }

    let git_dir_path = String::from_utf8_lossy(&git_dir.stdout).trim().to_string();
    let merge_head = Path::new(&git_dir_path).join("MERGE_HEAD");

    Ok(merge_head.exists())
}

pub fn merge_abort() -> GitResult<()> {
    Git::new("merge")
        .arg("--abort")
        .context("Failed to abort merge")
        .run()
}

pub fn has_diverged(branch: &str) -> GitResult<bool> {
    use crate::status::branch_status;

    let status = branch_status(branch)?;

    // Has diverged if we have both local commits (ahead) and remote commits (behind)
    Ok(status.ahead_count > 0 && status.behind_count > 0)
}

pub fn is_shared_branch(branch: &str) -> GitResult<bool> {
    use crate::status::branch_status;

    let status = branch_status(branch)?;

    // Branch is shared if it has an upstream branch
    // This means it exists on the remote and could be used by others
    Ok(status.upstream_branch.is_some())
}

pub fn ahead_behind(base: &str, compare: &str) -> GitResult<(i32, i32)> {
    let res = Command::new("git")
        .arg("rev-list")
        .arg("--left-right")
        .arg("--count")
        .arg(format!("{base}...{compare}"))
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

pub fn has_remote_branch(branch: &str) -> GitResult<bool> {
    let remotes = Git::new("remote")
        .context("Failed to list remotes")
        .output()?;
    Ok(remotes.lines().any(|r| r.trim() == branch))
}

pub fn current_upstream() -> GitResult<String> {
    Git::new("rev-parse")
        .args(["--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .context("Failed to get upstream branch")
        .output()
}

#[derive(Debug, Clone, Default)]
pub struct ChangedFile {
    pub name: String,
    pub additions: usize,
    pub deletions: usize,
}

/// Return all changed files relative to HEAD with additions and deletions.
pub fn changed_files() -> GitResult<Vec<ChangedFile>> {
    // Gather tracked changes (staged + unstaged) vs HEAD
    let diff_output = Git::new("diff")
        .args(["--numstat", "HEAD"]) // includes both staged and unstaged changes vs HEAD
        .context("Failed to get changed files")
        .output()?;

    let mut files = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for line in diff_output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }
        let additions = parts[0].parse::<usize>().unwrap_or(0);
        let deletions = parts[1].parse::<usize>().unwrap_or(0);
        let name = parts[2].to_string();
        seen.insert(name.clone());
        files.push(ChangedFile {
            name,
            additions,
            deletions,
        });
    }

    // Also include untracked files (not shown by `git diff`)
    // We approximate additions as the file's line count; ensure at least 1 so they render as "added".
    if let Ok(untracked) = Git::new("ls-files")
        .args(["--others", "--exclude-standard"]) // human-readable (newline separated)
        .output_lines()
    {
        for name in untracked
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            if seen.contains(&name) {
                continue;
            }

            let additions = fs::read_to_string(&name)
                .map(|s| {
                    let count = s.lines().count();
                    if count == 0 { 1 } else { count }
                })
                .unwrap_or(1);

            files.push(ChangedFile {
                name,
                additions,
                deletions: 0,
            });
        }
    }

    Ok(files)
}

/// Get total additions and deletions for the last commit.
pub fn last_commit_additions_deletions() -> GitResult<(usize, usize)> {
    // Use git log to emit only numstat lines for the last commit
    let output = Git::new("log")
        .args(["-1", "--numstat", "--format="])
        .context("Failed to get last commit stats")
        .output()?;

    let mut additions: usize = 0;
    let mut deletions: usize = 0;

    for line in output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }
        additions = additions.saturating_add(parts[0].parse::<usize>().unwrap_or(0));
        deletions = deletions.saturating_add(parts[1].parse::<usize>().unwrap_or(0));
    }

    Ok((additions, deletions))
}
