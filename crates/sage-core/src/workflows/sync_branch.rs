use anyhow::Result;
use colored::Colorize;
use sage_git::{
    branch::{get_current, get_default_branch, merge, pull, push, switch},
    repo::fetch_remote,
    status::status,
};
use sage_graph::SageGraph;

use crate::CliOutput;

pub struct SyncBranchOpts {
    pub continue_: bool,
    pub abort: bool,
    pub onto: Option<String>,
}

pub fn sync_branch(cli: &CliOutput) -> Result<()> {
    let current_branch = get_current()?;
    let mut parent = get_default_branch()?;
    let status = status()?;

    // Load the graph
    let graph = SageGraph::load_or_default()?;
    if graph.tracks(&current_branch) {
        // This will make it easier, as we are tracking the branch.
        if let Some(branch_info) = graph.info(&current_branch) {
            parent = branch_info.parent.to_string();
        }
    }

    cli.step_start(&format!("Syncing branch with parent '{parent}'"));
    // Fetching the remote
    fetch_remote()?;
    cli.step_success("Fetched origin", None);

    // Switch to the parent branch
    switch(&parent, false)?;

    let parent_status = status()?;

    if parent_status.needs_pull() {
        // Pull the branch
        cli.step_start("Pulling parent");
        pull()?;
        cli.step_success("Parent updated", None);
    }

    // We make the concious decision to not push unsynced changes on the parent to remote.

    // We will now switch back to the current branch
    switch(&current_branch, false)?;

    if status.needs_pull() {
        // We will now pull the changes for this branch
        cli.step_start("Pulling remote");
        pull()?;
        cli.step_success("Pulled remote", None);
    }

    if status.needs_push() {
        // We will now push the changes for this branch
        let spinner = cli.spinner("Pushing unsynced changes");
        push(&current_branch, false)?;
        spinner.finish_success("Synced changes with remote", None);
    }

    // We will now merge in the changes from the parent branch
    cli.step_start("Merging parent");
    merge(&parent)?;
    // TODO: Should get the latest commit hash
    cli.step_success("Merged parent into branch", None);

    // Refresh the status
    let status = status()?;

    if status.needs_push() {
        // We will now push the changes for this branch
        let spinner = cli.spinner("Pushing latest to remote");
        push(&current_branch, false)?;
        spinner.finish_success("Pushed to remote", Some(&current_branch.dimmed()));
    }

    Ok(())
}
