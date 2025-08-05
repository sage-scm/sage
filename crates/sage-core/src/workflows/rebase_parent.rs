use sage_git::branch::{get_current, merge, pull, push, switch};
use sage_git::rebase::rebase;
use sage_git::repo::fetch_remote;
use sage_git::status::branch_status;
use crate::CliOutput;

pub fn rebase_onto_parent(
    cli: &CliOutput,
    branch: &str,
    parent: &str,
    use_rebase: bool,
    force_push: bool
) -> anyhow::Result<()> {
    cli.step_start(&format!(
        "Syncing '{}' with '{}' ({})",
        branch,
        parent,
        if use_rebase { "rebase" } else { "merge" }
    ));

    fetch_remote()?;

    let original_branch = get_current()?;
    let needs_switch_back = original_branch != branch;

    if needs_switch_back {
        switch(branch, false)?;
    }

    let branch_state = branch_status(branch)?;
    if branch_state.needs_pull() {
        pull()?;
    }

    ensure_parent_updated(parent)?;

    if use_rebase {
        rebase(parent, false, true)?;
    } else {
        merge(parent)?;
    }

    if branch_state.needs_push() || use_rebase || force_push {
        let force = use_rebase || force_push;
        push(branch, force)?;
    }

    if needs_switch_back {
        switch(&original_branch, false)?;
    }

    cli.step_success(&format!("Synced '{}'", branch), None);
    Ok(())
}

fn ensure_parent_updated(parent: &str) -> anyhow::Result<()> {
    let current = get_current()?;

    switch(parent, false)?;
    pull()?;

    if current != parent {
        switch(&current, false)?;
    }

    Ok(())
}