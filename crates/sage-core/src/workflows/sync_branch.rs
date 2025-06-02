use anyhow::{bail, Result};
use sage_git::{
    branch::{get_current, get_default_branch, is_clean, merge, pull, push, switch},
    rebase::rebase,
    repo::fetch_remote,
    status::status,
};
use sage_graph::SageGraph;

use crate::CliOutput;

/// Options for syncing a branch
#[derive(Debug, Default)]
pub struct SyncBranchOpts {
    /// Continue an interrupted sync operation
    pub continue_: bool,
    /// Abort the current sync operation
    pub abort: bool,
    /// Rebase instead of merge when syncing
    pub rebase: bool,
    /// Force push after successful sync
    pub force_push: bool,
    /// Parent branch to sync with (defaults to tracked parent or default branch)
    pub parent: Option<String>,
}

/// State of the sync operation
#[derive(Debug)]
struct SyncState {
    original_branch: String,
    parent_branch: String,
    needs_push: bool,
    rebase: bool,
}

pub fn sync_branch(cli: &CliOutput, opts: &SyncBranchOpts) -> Result<()> {
    // Handle continue/abort flags
    if opts.continue_ || opts.abort {
        return handle_special_ops(cli, opts);
    }

    // Get current repository state
    let current_branch = get_current()?;
    let repo_status = status()?;

    // Validate repository state
    if !is_clean()? {
        bail!(
            "Working directory is not clean. Please commit or stash your changes before syncing."
        );
    }

    // Determine parent branch
    let parent_branch = determine_parent_branch(&current_branch, &opts.parent)?;

    if current_branch == parent_branch {
        return sync_default_branch(cli, &current_branch);
    }

    // Set up sync state
    let state = SyncState {
        original_branch: current_branch.clone(),
        parent_branch: parent_branch.clone(),
        needs_push: repo_status.needs_push(),
        rebase: opts.rebase,
    };

    // Perform the sync
    sync_with_parent(cli, state)
}

/// Handle continue/abort operations
fn handle_special_ops(cli: &CliOutput, opts: &SyncBranchOpts) -> Result<()> {
    if opts.continue_ {
        cli.step_start("Continuing sync operation");
        cli.step_success("Sync operation continued", None);
    } else if opts.abort {
        cli.step_start("Aborting sync operation");
        cli.step_success("Sync operation aborted", None);
    }
    Ok(())
}

/// Sync the default branch (special case)
fn sync_default_branch(cli: &CliOutput, branch: &str) -> Result<()> {
    cli.step_start(&format!("Syncing default branch '{}'", branch));

    // Fetch latest changes
    cli.step_start("Fetching latest changes");
    fetch_remote()?;

    // Pull with rebase to keep history clean
    cli.step_start("Pulling latest changes");
    pull()?;

    cli.step_success("Default branch synced", None);
    Ok(())
}

/// Determine the parent branch to sync with
fn determine_parent_branch(
    current_branch: &str,
    explicit_parent: &Option<String>,
) -> Result<String> {
    if let Some(parent) = explicit_parent {
        return Ok(parent.clone());
    }

    // Try to get parent from graph if branch is tracked
    let graph = SageGraph::load_or_default()?;
    if graph.tracks(current_branch) {
        if let Some(branch_info) = graph.info(current_branch) {
            return Ok(branch_info.parent.to_string());
        }
    }

    // Fall back to default branch
    get_default_branch()
}

/// Core sync logic
fn sync_with_parent(cli: &CliOutput, state: SyncState) -> Result<()> {
    cli.step_start(&format!(
        "Syncing branch '{}' with parent '{}' using {}",
        state.original_branch,
        state.parent_branch,
        if state.rebase { "rebase" } else { "merge" }
    ));

    // Fetch latest changes from remote
    cli.step_start("Fetching latest changes from remote");
    fetch_remote()?;

    // Switch to parent branch and update it
    cli.step_start(&format!("Updating parent branch '{}'", state.parent_branch));
    switch(&state.parent_branch, false)?;
    pull()?;

    // Switch back to feature branch
    cli.step_start(&format!("Updating branch '{}'", state.original_branch));
    switch(&state.original_branch, false)?;

    // Update feature branch from parent
    if state.rebase {
        cli.step_start("Rebasing onto parent");
        rebase(&state.parent_branch, false, false)?;
    } else {
        cli.step_start("Merging parent");
        merge(&state.parent_branch)?;
    }

    // Push changes if needed
    if state.needs_push || state.rebase {
        cli.step_start("Pushing changes to remote");
        push(&state.original_branch, state.rebase)?;
    }

    cli.step_success("Sync completed successfully", None);
    Ok(())
}
