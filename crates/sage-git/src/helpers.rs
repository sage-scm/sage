use crate::prelude::{Git, GitResult};
use anyhow::bail;

pub fn get_commit_hash(reference: &str) -> GitResult<String> {
    Git::new("rev-parse")
        .arg(reference)
        .context("Failed to get commit hash")
        .output()
}

pub fn get_short_commit_hash(reference: &str) -> GitResult<String> {
    Git::new("rev-parse")
        .args(["--short", reference])
        .context("Failed to get short commit hash")
        .output()
}

pub fn get_commit_files(commit_id: &str) -> GitResult<Vec<String>> {
    let output = Git::new("diff-tree")
        .args(["--no-commit-id", "--name-only", "-r", commit_id])
        .context("Failed to get commit files")
        .output()?;

    Ok(output.lines().map(|s| s.to_string()).collect())
}

pub fn get_files_changed_between(from: &str, to: &str) -> GitResult<Vec<String>> {
    let output = Git::new("diff")
        .args(["--name-only", &format!("{from}..{to}")])
        .context("Failed to get changed files")
        .output()?;

    Ok(output.lines().map(|s| s.to_string()).collect())
}

pub fn cherry_pick(commit: &str, no_commit: bool) -> GitResult<()> {
    let mut git = Git::new("cherry-pick");

    if no_commit {
        git = git.arg("--no-commit");
    }

    git.arg(commit)
        .context("Failed to cherry-pick commit")
        .run()
}

pub fn cherry_pick_range(from: &str, to: &str) -> GitResult<()> {
    Git::new("cherry-pick")
        .arg(&format!("{from}..{to}"))
        .context("Failed to cherry-pick range")
        .run()
}

pub fn reset_hard(reference: &str) -> GitResult<()> {
    Git::new("reset")
        .args(["--hard", reference])
        .context("Failed to reset hard")
        .run()
}

pub fn reset_soft(reference: &str) -> GitResult<()> {
    Git::new("reset")
        .args(["--soft", reference])
        .context("Failed to reset soft")
        .run()
}

pub fn reset_mixed(reference: &str) -> GitResult<()> {
    Git::new("reset")
        .args(["--mixed", reference])
        .context("Failed to reset mixed")
        .run()
}

pub fn clean_untracked(force: bool, directories: bool) -> GitResult<()> {
    let mut git = Git::new("clean");

    if force {
        git = git.arg("-f");
    }

    if directories {
        git = git.arg("-d");
    }

    git.context("Failed to clean untracked files").run()
}

pub fn get_merge_base(branch1: &str, branch2: &str) -> GitResult<String> {
    Git::new("merge-base")
        .args([branch1, branch2])
        .context("Failed to get merge base")
        .output()
}

pub fn get_common_ancestor(branches: &[&str]) -> GitResult<String> {
    if branches.is_empty() {
        bail!("No branches provided");
    }

    let mut git = Git::new("merge-base");
    for branch in branches {
        git = git.arg(branch);
    }

    git.context("Failed to get common ancestor").output()
}

pub fn create_tag(name: &str, message: Option<&str>, reference: Option<&str>) -> GitResult<()> {
    let mut git = Git::new("tag");

    if let Some(msg) = message {
        git = git.args(["-a", name, "-m", msg]);
    } else {
        git = git.arg(name);
    }

    if let Some(ref_) = reference {
        git = git.arg(ref_);
    }

    git.context("Failed to create tag").run()
}

pub fn delete_tag(name: &str) -> GitResult<()> {
    Git::new("tag")
        .args(["-d", name])
        .context("Failed to delete tag")
        .run()
}

pub fn list_tags(pattern: Option<&str>) -> GitResult<Vec<String>> {
    let mut git = Git::new("tag").arg("-l");

    if let Some(p) = pattern {
        git = git.arg(p);
    }

    git.context("Failed to list tags").output_lines()
}

pub fn push_tag(tag: &str, force: bool) -> GitResult<()> {
    let mut git = Git::new("push").arg("origin").arg(tag);

    if force {
        git = git.arg("--force");
    }

    git.context("Failed to push tag").run()
}

pub fn fetch_tags() -> GitResult<()> {
    Git::new("fetch")
        .args(["--tags", "--all"])
        .context("Failed to fetch tags")
        .run()
}

pub fn get_remote_url(remote: &str) -> GitResult<String> {
    Git::new("remote")
        .args(["get-url", remote])
        .context("Failed to get remote URL")
        .output()
}

pub fn add_remote(name: &str, url: &str) -> GitResult<()> {
    Git::new("remote")
        .args(["add", name, url])
        .context("Failed to add remote")
        .run()
}

pub fn remove_remote(name: &str) -> GitResult<()> {
    Git::new("remote")
        .args(["remove", name])
        .context("Failed to remove remote")
        .run()
}

pub fn list_remotes() -> GitResult<Vec<String>> {
    Git::new("remote")
        .context("Failed to list remotes")
        .output_lines()
}

pub fn show_commit(commit: &str, format: Option<&str>) -> GitResult<String> {
    let mut git = Git::new("show");

    if let Some(fmt) = format {
        git = git.arg(&format!("--format={fmt}"));
    }

    git.arg(commit).context("Failed to show commit").output()
}

pub fn get_commit_message(commit: &str) -> GitResult<String> {
    Git::new("log")
        .args(["-1", "--pretty=%B", commit])
        .context("Failed to get commit message")
        .output()
}

pub fn get_commit_author(commit: &str) -> GitResult<String> {
    Git::new("log")
        .args(["-1", "--pretty=%an <%ae>", commit])
        .context("Failed to get commit author")
        .output()
}

pub fn get_commit_date(commit: &str) -> GitResult<String> {
    Git::new("log")
        .args(["-1", "--pretty=%aI", commit])
        .context("Failed to get commit date")
        .output()
}

pub fn count_commits(from: Option<&str>, to: Option<&str>) -> GitResult<usize> {
    let mut git = Git::new("rev-list").arg("--count");

    if let (Some(f), Some(t)) = (from, to) {
        git = git.arg(&format!("{f}..{t}"));
    } else if let Some(t) = to {
        git = git.arg(t);
    } else {
        git = git.arg("HEAD");
    }

    let output = git.context("Failed to count commits").output()?;

    output
        .trim()
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse commit count: {}", e))
}

pub fn is_ancestor(ancestor: &str, descendant: &str) -> GitResult<bool> {
    Git::new("merge-base")
        .args(["--is-ancestor", ancestor, descendant])
        .success()
}

pub fn create_worktree(path: &str, branch: Option<&str>) -> GitResult<()> {
    let mut git = Git::new("worktree").arg("add").arg(path);

    if let Some(b) = branch {
        git = git.arg(b);
    }

    git.context("Failed to create worktree").run()
}

pub fn remove_worktree(path: &str) -> GitResult<()> {
    Git::new("worktree")
        .args(["remove", path])
        .context("Failed to remove worktree")
        .run()
}

pub fn list_worktrees() -> GitResult<Vec<String>> {
    Git::new("worktree")
        .arg("list")
        .context("Failed to list worktrees")
        .output_lines()
}

pub fn bisect_start(bad: Option<&str>, good: Option<&str>) -> GitResult<()> {
    let mut git = Git::new("bisect").arg("start");

    if let Some(b) = bad {
        git = git.arg(b);
    }

    if let Some(g) = good {
        git = git.arg(g);
    }

    git.context("Failed to start bisect").run()
}

pub fn bisect_good(commit: Option<&str>) -> GitResult<()> {
    let mut git = Git::new("bisect").arg("good");

    if let Some(c) = commit {
        git = git.arg(c);
    }

    git.context("Failed to mark as good").run()
}

pub fn bisect_bad(commit: Option<&str>) -> GitResult<()> {
    let mut git = Git::new("bisect").arg("bad");

    if let Some(c) = commit {
        git = git.arg(c);
    }

    git.context("Failed to mark as bad").run()
}

pub fn bisect_reset() -> GitResult<()> {
    Git::new("bisect")
        .arg("reset")
        .context("Failed to reset bisect")
        .run()
}

pub fn get_file_at_revision(file: &str, revision: &str) -> GitResult<String> {
    Git::new("show")
        .arg(&format!("{revision}:{file}"))
        .context("Failed to get file at revision")
        .output()
}

pub fn blame(file: &str, line_range: Option<(usize, usize)>) -> GitResult<String> {
    let mut git = Git::new("blame");

    if let Some((start, end)) = line_range {
        git = git.arg(&format!("-L{start},{end}"));
    }

    git.arg(file).context("Failed to blame file").output()
}

pub fn get_branch_contains(commit: &str) -> GitResult<Vec<String>> {
    Git::new("branch")
        .args(["--contains", commit])
        .context("Failed to get branches containing commit")
        .output_lines()
}

pub fn is_empty_commit(commit: &str) -> GitResult<bool> {
    let output = Git::new("diff-tree")
        .args(["--no-commit-id", "--name-only", "-r", commit])
        .output()?;

    Ok(output.trim().is_empty())
}

pub fn squash_commits(from: &str, message: Option<&str>) -> GitResult<()> {
    reset_soft(from)?;

    let mut git = Git::new("commit");

    if let Some(msg) = message {
        git = git.args(["-m", msg]);
    } else {
        git = git.arg("--edit");
    }

    git.context("Failed to squash commits").run()
}

pub fn verify_commit_signature(commit: &str) -> GitResult<bool> {
    Git::new("verify-commit").arg(commit).success()
}

pub fn get_ref_type(reference: &str) -> GitResult<String> {
    Git::new("cat-file")
        .args(["-t", reference])
        .context("Failed to get reference type")
        .output()
}
