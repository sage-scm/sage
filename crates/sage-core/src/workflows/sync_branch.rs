use anyhow::{Result, bail};
use sage_git::{
    branch::{
        get_current, get_default_branch, is_clean, is_merge_in_progress, merge, merge_abort, pull,
        push, switch,
    },
    commit::commit,
    config::{is_up_to_date_with_upstream, should_branch_rebase},
    rebase::{is_rebase_in_progress, rebase, rebase_abort, rebase_continue},
    repo::{fetch_remote, has_conflicts},
    stash::{stash_pop, stash_push},
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
    /// Automatically stash and unstash changes if needed
    pub autostash: bool,
    /// Only sync the current branch, not the entire stack
    pub no_stack: bool,
}

/// State of the sync operation
#[derive(Debug)]
struct SyncState {
    original_branch: String,
    parent_branch: String,
    needs_pull: bool,
    needs_push: bool,
    rebase: bool,
}

fn should_use_rebase(
    branch: &str,
    graph: &SageGraph,
    explicit_rebase: Option<bool>,
) -> Result<bool> {
    if let Some(rebase) = explicit_rebase {
        return Ok(rebase);
    }

    if graph.stack_of(branch).is_some() {
        return Ok(true);
    }

    if let Some(rebase) = should_branch_rebase(branch)? {
        return Ok(rebase);
    }

    if is_up_to_date_with_upstream(branch)? {
        return Ok(false);
    }

    Ok(true)
}

fn get_branch_chain(graph: &SageGraph, start_branch: &str) -> Vec<String> {
    let mut chain = vec![start_branch.to_string()];
    let mut current = start_branch;

    while let Some(info) = graph.info(current) {
        if info.parent == current || info.parent.is_empty() {
            break;
        }
        chain.push(info.parent.clone());
        current = &info.parent;
    }

    chain
}

pub fn sync_branch(cli: &CliOutput, opts: &SyncBranchOpts) -> Result<()> {
    // Handle continue/abort flags
    if opts.continue_ || opts.abort {
        return handle_special_ops(cli, opts);
    }

    // Get current repository state
    let current_branch = get_current()?;
    let repo_status = status()?;

    // Handle autostash if enabled
    let mut stashed = false;
    if !is_clean()? {
        // Check if we're in the middle of a rebase or merge
        if is_rebase_in_progress()? {
            bail!(
                "Rebase in progress. Please complete it with 'sage sync --continue' or abort with 'sage sync --abort'"
            );
        }
        if is_merge_in_progress()? {
            bail!(
                "Merge in progress. Please complete it with 'sage sync --continue' or abort with 'sage sync --abort'"
            );
        }

        if opts.autostash {
            cli.step_start("Stashing local changes");
            stash_push(Some("sage sync autostash"))?;
            stashed = true;
            cli.step_success("Changes stashed", Some("will restore after sync"));
        } else {
            bail!(
                "Working directory is not clean. Please commit or stash your changes before syncing, or use --autostash."
            );
        }
    }

    // Load the graph to check if we're in a stack
    let graph = SageGraph::load_or_default()?;
    let is_in_stack = graph.stack_of(&current_branch).is_some();

    // Determine if we should sync the stack
    let should_sync_stack = is_in_stack && !opts.no_stack;

    if should_sync_stack {
        // Get the chain of branches from current up to root
        let branch_chain = get_branch_chain(&graph, &current_branch);
        let stack_size = branch_chain.len();

        if stack_size > 1 {
            cli.step_success(
                &format!("Stack detected: {}", branch_chain.join(" → ")),
                None,
            );
        }

        // We need to sync from root to current (reverse order)
        let mut branches_to_sync: Vec<String> = branch_chain.into_iter().rev().collect();

        // If the root is the default branch, we'll handle it specially
        let default_branch = get_default_branch()?;
        if let Some(root) = branches_to_sync.first() {
            if root == &default_branch {
                // Sync default branch first
                sync_default_branch(cli, &default_branch)?;
                branches_to_sync.remove(0);
            }
        }

        // Now sync each branch in the stack
        for (i, branch) in branches_to_sync.iter().enumerate() {
            // Skip if it's the current branch (we'll handle it below)
            if branch == &current_branch {
                continue;
            }

            cli.step_start(&format!(
                "[{}/{}] Syncing '{}'",
                i + 1,
                branches_to_sync.len(),
                branch
            ));

            // Switch to the branch
            switch(branch, false)?;

            // Determine parent for this branch
            let parent = if i == 0 {
                // First branch after default, parent is default
                default_branch.clone()
            } else {
                // Parent is the previous branch in the chain
                branches_to_sync[i - 1].clone()
            };

            // Determine whether to use rebase or merge for this branch
            let use_rebase =
                should_use_rebase(branch, &graph, if opts.rebase { Some(true) } else { None })?;

            // Sync this branch with its parent
            let branch_state = SyncState {
                original_branch: branch.clone(),
                parent_branch: parent.clone(),
                needs_push: status()?.needs_push(),
                rebase: use_rebase,
            };

            match sync_with_parent_internal(cli, &branch_state, opts.force_push, false) {
                Ok(_) => {
                    cli.step_success(&format!("Synced '{}' with '{}'", branch, parent), None);
                }
                Err(e) => {
                    cli.step_error(&format!("Failed to sync '{}'", branch), &e.to_string());
                    if is_rebase_in_progress()? || is_merge_in_progress()? {
                        cli.warning("Resolve conflicts and run 'sage sync --continue'");
                    }
                    return Err(e);
                }
            }
        }

        // Switch back to original branch
        if get_current()? != current_branch {
            cli.step_start(&format!("Returning to '{}'", current_branch));
            switch(&current_branch, false)?;
            cli.step_success(&format!("Back on '{}'", current_branch), None);
        }
    }

    // Determine parent branch for current branch
    let parent_branch = determine_parent_branch(&current_branch, &opts.parent)?;

    if current_branch == parent_branch {
        return sync_default_branch(cli, &current_branch);
    }

    // Determine whether to use rebase or merge
    let use_rebase = should_use_rebase(
        &current_branch,
        &graph,
        if opts.rebase { Some(true) } else { None },
    )?;

    // Set up sync state for current branch
    let state = SyncState {
        original_branch: current_branch.clone(),
        parent_branch: parent_branch.clone(),
        needs_pull: repo_status.needs_pull(),
        needs_push: repo_status.needs_push(),
        rebase: use_rebase,
    };

    // Perform the sync for current branch
    let result = sync_with_parent(cli, &state, opts.force_push);

    // Handle unstashing if we stashed earlier
    if stashed {
        match result {
            Ok(_) => {
                cli.step_start("Restoring stashed changes");
                match stash_pop() {
                    Ok(_) => cli.step_success("Stashed changes restored", None),
                    Err(e) => {
                        cli.step_error("Failed to restore stashed changes", &e.to_string());
                        cli.warning("Your changes are still stashed. Use 'git stash pop' to restore them manually");
                    }
                }
            }
            Err(_) => {
                cli.warning("Note: Your changes are still stashed. They will be restored after resolving conflicts");
            }
        }
    }

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            // Check if we failed due to conflicts
            if e.to_string().contains("conflicts")
                || e.to_string().contains("CONFLICT")
                || is_rebase_in_progress()?
                || is_merge_in_progress()?
            {
                cli.step_error("Sync encountered conflicts", "See details above");
                cli.warning(
                    "Resolve conflicts and run 'sage sync --continue' to complete the sync",
                );
                cli.warning("Or run 'sage sync --abort' to cancel the sync operation");
            }
            Err(e)
        }
    }
}

/// Handle continue/abort operations
fn handle_special_ops(cli: &CliOutput, opts: &SyncBranchOpts) -> Result<()> {
    // Check if we're in the middle of a rebase or merge
    let in_rebase = is_rebase_in_progress()?;
    let in_merge = is_merge_in_progress()?;

    if opts.continue_ {
        if !in_rebase && !in_merge {
            bail!("No sync operation in progress to continue");
        }

        cli.step_start("Checking for conflicts");

        if has_conflicts()? {
            cli.step_error("Unresolved conflicts found", "please resolve all conflicts");
            bail!(
                "Please resolve all conflicts and stage changes with 'git add' before continuing"
            );
        }

        cli.step_success("No conflicts found", Some("ready to continue"));

        if in_rebase {
            cli.step_start("Continuing rebase");
            rebase_continue()?;
            cli.step_success("Rebase continued", None);
        } else if in_merge {
            cli.step_start("Creating merge commit");

            let current_branch = get_current()?;
            let parent_branch = determine_parent_branch(&current_branch, &None)?;
            let merge_message =
                format!("merge: sync '{}' with '{}'", current_branch, parent_branch);

            commit(&merge_message)?;

            cli.step_success("Merge commit created", None);
        }

        let current_branch = get_current()?;
        let graph = SageGraph::load_or_default()?;
        let use_rebase = should_use_rebase(
            &current_branch,
            &graph,
            if opts.rebase { Some(true) } else { None },
        )?;

        if use_rebase || opts.force_push {
            cli.step_start("Pushing changes to remote");
            push(&current_branch, true)?;
            cli.step_success("Pushed to remote", Some("--force"));
        }
    } else if opts.abort {
        if !in_rebase && !in_merge {
            bail!("No sync operation in progress to abort");
        }

        cli.step_start("Aborting sync operation");

        if in_rebase {
            rebase_abort()?;
            cli.step_success("Rebase aborted", None);
        } else if in_merge {
            merge_abort()?;
            cli.step_success("Merge aborted", None);
        }
    }
    Ok(())
}

fn sync_default_branch(cli: &CliOutput, branch: &str) -> Result<()> {
    cli.step_start(&format!("Syncing default branch '{}'", branch));

    fetch_remote()?;
    pull()?;

    cli.step_success(&format!("Default branch '{}' synced", branch), None);
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

fn sync_with_parent_internal(
    _cli: &CliOutput,
    state: &SyncState,
    force_push: bool,
    _show_output: bool,
) -> Result<()> {
    fetch_remote()?;

    // Pulling on this branch first.
    if state.needs_pull {
        pull()?;
    }

    switch(&state.parent_branch, false)?;
    pull()?;

    switch(&state.original_branch, false)?;

    if state.rebase {
        rebase(&state.parent_branch, false, true)?;
    } else {
        merge(&state.parent_branch)?;
    }

    if state.needs_push || state.rebase || force_push {
        let force = state.rebase || force_push;
        push(&state.original_branch, force)?;
    }

    Ok(())
}

fn sync_with_parent(cli: &CliOutput, state: &SyncState, force_push: bool) -> Result<()> {
    cli.step_start(&format!(
        "Syncing '{}' with '{}' ({})",
        state.original_branch,
        state.parent_branch,
        if state.rebase { "rebase" } else { "merge" }
    ));

    fetch_remote()?;
    switch(&state.parent_branch, false)?;
    pull()?;
    switch(&state.original_branch, false)?;

    if state.rebase {
        rebase(&state.parent_branch, false, true)?;
    } else {
        merge(&state.parent_branch)?;
    }

    if state.needs_push || state.rebase || force_push {
        let force = state.rebase || force_push;
        push(&state.original_branch, force)?;
    }

    cli.step_success(&format!("Synced '{}'", state.original_branch), None);
    Ok(())
}
