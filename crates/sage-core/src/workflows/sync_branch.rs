use anyhow::Result;
use sage_git::branch::{get_current, get_default_branch};
use sage_graph::SageGraph;

pub struct SyncBranchOpts {
    pub continue_: bool,
    pub abort: bool,
    pub onto: Option<String>,
}

pub fn sync_branch() -> Result<()> {
    let current_branch = get_current()?;
    let mut parent = get_default_branch()?;
    let status = sage_git::status::status()?;

    // Load the graph
    let graph = SageGraph::load_or_default()?;
    if graph.tracks(&current_branch) {
        // This will make it easier, as we are tracking the branch.
        if let Some(branch_info) = graph.info(&current_branch) {
            parent = branch_info.parent.to_string();
        }
    }

    println!("●  Syncing branch with parent '{parent}'");

    // Fetching the remote
    sage_git::repo::fetch_remote()?;
    println!("●  Fetching origin       ✔");

    // Switch to the parent branch
    sage_git::branch::switch(&parent, false)?;

    let parent_status = sage_git::status::status()?;

    if parent_status.needs_pull() {
        // Pull the branch
        sage_git::branch::pull()?;
        println!("●  Pulling parent        ✔");
    }

    // We make the concious decision to not push unsynced changes on the parent to remote.

    // We will now switch back to the current branch
    sage_git::branch::switch(&current_branch, false)?;

    if status.needs_pull() {
        // We will now pull the changes for this branch
        sage_git::branch::pull()?;
        println!("●  Pulling origin        ✔");
    }

    if status.needs_push() {
        // We will now push the changes for this branch
        sage_git::branch::push(&current_branch, false)?;
        println!("●  Push unsynced changes           ✔");
    }

    // We will now merge in the changes from the parent branch
    sage_git::branch::merge(&parent)?;
    println!("●  Merging parent       ✔");

    // Refresh the status
    let status = sage_git::status::status()?;

    if status.needs_push() {
        // We will now push the changes for this branch
        sage_git::branch::push(&current_branch, false)?;
        println!("●  Push origin/{}        ✔", current_branch);
    }

    Ok(())
}
