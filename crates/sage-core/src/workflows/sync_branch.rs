use anyhow::{Result, bail};
use sage_git::{
    branch::{
        get_current, get_default_branch, has_diverged, is_merge_in_progress,
        is_shared_branch, merge_abort, pull, push,
    },
    commit::commit,
    config::should_branch_rebase,
    rebase::{is_rebase_in_progress, rebase_abort, rebase_continue},
    repo::{fetch_remote, has_conflicts},
    stash::stash_pop,
    status::branch_status,
};
use sage_graph::SageGraph;

use crate::workflows::stash_dirty::stash_if_dirty;
use crate::{CliOutput, rebase_onto_parent};

#[derive(Debug, Default)]
pub struct SyncBranchOpts {
    pub continue_: bool,
    pub abort: bool,
    pub rebase: bool,
    pub force_push: bool,
    pub parent: Option<String>,
    pub autostash: bool,
    pub no_stack: bool,
}

pub fn sync_branch(cli: &CliOutput, opts: &SyncBranchOpts) -> Result<()> {
    if opts.continue_ || opts.abort {
        return handle_sync_interrupt(cli, opts);
    }

    let current_branch = get_current()?;
    let graph = SageGraph::load_or_default()?;

    let stashed = stash_if_dirty(cli, opts)?;

    let result = if should_sync_entire_stack(&current_branch, &graph, opts) {
        sync_stack(cli, &current_branch, &graph, opts)
    } else {
        sync_single_branch(cli, &current_branch, &graph, opts)
    };

    restore_stash_if_needed(cli, stashed, &result);

    handle_sync_result(cli, result)
}

fn restore_stash_if_needed(cli: &CliOutput, stashed: bool, result: &Result<()>) {
    if !stashed {
        return;
    }

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

fn sync_single_branch(
    cli: &CliOutput,
    current_branch: &str,
    graph: &SageGraph,
    opts: &SyncBranchOpts,
) -> Result<()> {
    let parent_branch = find_parent_branch(current_branch, &opts.parent)?;

    if current_branch == parent_branch {
        return ensure_branch_updated(cli, current_branch);
    }

    let branch_state = branch_status(current_branch)?;

    if branch_state.needs_pull() && !branch_state.needs_push() {
        cli.step_start(&format!("Pulling latest changes for '{current_branch}'"));
        pull()?;
        cli.step_success("Branch updated from remote", None);
        return Ok(());
    }

    if !branch_state.needs_pull() && branch_state.needs_push() {
        cli.step_start(&format!("Pushing changes for '{current_branch}'"));
        push(current_branch, false)?;
        cli.step_success("Changes pushed to remote", None);
        return Ok(());
    }

    let parent_needs_update = branch_needs_update(&parent_branch)?;
    if !parent_needs_update && !branch_state.needs_pull() {
        cli.step_success(&format!("'{current_branch}' is already up to date"), None);
        return Ok(());
    }

    let use_rebase = decide_sync_method(current_branch, graph, opts.rebase)?;

    rebase_onto_parent(
        cli,
        current_branch,
        &parent_branch,
        use_rebase,
        opts.force_push,
    )
}

pub fn ensure_branch_updated(cli: &CliOutput, branch: &str) -> Result<()> {
    cli.step_start(&format!("Updating '{branch}'"));

    fetch_remote()?;

    let status = branch_status(branch)?;

    if status.needs_pull() {
        pull()?;
        cli.step_success(&format!("'{branch}' updated from remote"), None);
    } else {
        cli.step_success(&format!("'{branch}' already up to date"), None);
    }

    Ok(())
}

fn branch_needs_update(branch: &str) -> Result<bool> {
    let status = branch_status(branch)?;
    Ok(status.needs_pull())
}

fn handle_sync_interrupt(cli: &CliOutput, opts: &SyncBranchOpts) -> Result<()> {
    let in_rebase = is_rebase_in_progress()?;
    let in_merge = is_merge_in_progress()?;

    if !in_rebase && !in_merge {
        bail!(
            "No sync operation in progress to {}",
            if opts.continue_ { "continue" } else { "abort" }
        );
    }

    if opts.continue_ {
        continue_sync(cli, in_rebase, in_merge, opts)
    } else {
        abort_sync(cli, in_rebase, in_merge)
    }
}

fn continue_sync(
    cli: &CliOutput,
    in_rebase: bool,
    in_merge: bool,
    opts: &SyncBranchOpts,
) -> Result<()> {
    cli.step_start("Checking for conflicts");

    if has_conflicts()? {
        cli.step_error("Unresolved conflicts found", "please resolve all conflicts");
        bail!("Please resolve all conflicts and stage changes with 'git add' before continuing");
    }

    cli.step_success("No conflicts found", Some("ready to continue"));

    if in_rebase {
        cli.step_start("Continuing rebase");
        rebase_continue()?;
        cli.step_success("Rebase continued", None);
    } else if in_merge {
        complete_merge(cli)?;
    }

    push_if_needed(cli, opts)?;
    Ok(())
}

fn complete_merge(cli: &CliOutput) -> Result<()> {
    cli.step_start("Creating merge commit");

    let current_branch = get_current()?;
    let parent_branch = find_parent_branch(&current_branch, &None)?;
    let merge_message = format!("merge: sync '{current_branch}' with '{parent_branch}'");

    commit(&merge_message)?;

    cli.step_success("Merge commit created", None);
    Ok(())
}

fn push_if_needed(cli: &CliOutput, opts: &SyncBranchOpts) -> Result<()> {
    let current_branch = get_current()?;
    let graph = SageGraph::load_or_default()?;
    let use_rebase = decide_sync_method(&current_branch, &graph, opts.rebase)?;

    if use_rebase || opts.force_push {
        cli.step_start("Pushing changes to remote");
        push(&current_branch, true)?;
        cli.step_success("Pushed to remote", Some("--force"));
    }
    Ok(())
}

fn abort_sync(cli: &CliOutput, in_rebase: bool, in_merge: bool) -> Result<()> {
    cli.step_start("Aborting sync operation");

    if in_rebase {
        rebase_abort()?;
        cli.step_success("Rebase aborted", None);
    } else if in_merge {
        merge_abort()?;
        cli.step_success("Merge aborted", None);
    }
    Ok(())
}

fn handle_sync_result(cli: &CliOutput, result: Result<()>) -> Result<()> {
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            if sync_has_conflicts(&e)? {
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

fn sync_has_conflicts(error: &anyhow::Error) -> Result<bool> {
    Ok(error.to_string().contains("conflicts")
        || error.to_string().contains("CONFLICT")
        || is_rebase_in_progress()?
        || is_merge_in_progress()?)
}

fn should_sync_entire_stack(branch: &str, graph: &SageGraph, opts: &SyncBranchOpts) -> bool {
    graph.stack_of(branch).is_some() && !opts.no_stack
}

fn sync_stack(
    cli: &CliOutput,
    current_branch: &str,
    graph: &SageGraph,
    opts: &SyncBranchOpts,
) -> Result<()> {
    let branch_chain = build_branch_chain_to_root(graph, current_branch);

    if branch_chain.len() > 1 {
        cli.step_success(
            &format!("Stack detected: {}", branch_chain.join(" → ")),
            None,
        );
    }

    let branches_to_sync = prepare_branches_for_sync(cli, branch_chain)?;

    for (i, branch) in branches_to_sync.iter().enumerate() {
        if branch == current_branch {
            continue;
        }

        cli.step_start(&format!(
            "[{}/{}] Syncing '{}'",
            i + 1,
            branches_to_sync.len(),
            branch
        ));

        let parent = if i == 0 {
            get_default_branch()?
        } else {
            branches_to_sync[i - 1].clone()
        };

        let use_rebase = decide_sync_method(branch, graph, opts.rebase)?;

        match rebase_onto_parent(cli, branch, &parent, use_rebase, opts.force_push) {
            Ok(_) => {
                cli.step_success(&format!("Synced '{branch}' with '{parent}'"), None);
            }
            Err(e) => {
                cli.step_error(&format!("Failed to sync '{branch}'"), &e.to_string());
                if is_rebase_in_progress()? || is_merge_in_progress()? {
                    cli.warning("Resolve conflicts and run 'sage sync --continue'");
                }
                return Err(e);
            }
        }
    }

    sync_single_branch(cli, current_branch, graph, opts)
}

fn build_branch_chain_to_root(graph: &SageGraph, start_branch: &str) -> Vec<String> {
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

fn prepare_branches_for_sync(cli: &CliOutput, branch_chain: Vec<String>) -> Result<Vec<String>> {
    let mut branches_to_sync: Vec<String> = branch_chain.into_iter().rev().collect();
    let default_branch = get_default_branch()?;

    if let Some(root) = branches_to_sync.first() {
        if root == &default_branch {
            ensure_branch_updated(cli, &default_branch)?;
            branches_to_sync.remove(0);
        }
    }

    Ok(branches_to_sync)
}

fn find_parent_branch(current_branch: &str, explicit_parent: &Option<String>) -> Result<String> {
    if let Some(parent) = explicit_parent {
        return Ok(parent.clone());
    }

    let graph = SageGraph::load_or_default()?;
    if graph.tracks(current_branch) {
        if let Some(branch_info) = graph.info(current_branch) {
            return Ok(branch_info.parent.to_string());
        }
    }

    get_default_branch()
}

fn decide_sync_method(branch: &str, graph: &SageGraph, force_rebase: bool) -> Result<bool> {
    if force_rebase {
        return Ok(true);
    }

    if graph.stack_of(branch).is_some() {
        return Ok(true);
    }

    if has_diverged(branch)? {
        return Ok(false);
    }

    if is_shared_branch(branch)? {
        return Ok(false);
    }

    if let Some(rebase) = should_branch_rebase(branch)? {
        return Ok(rebase);
    }

    Ok(true)
}
