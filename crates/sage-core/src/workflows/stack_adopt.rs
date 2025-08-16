use anyhow::{Result, bail};
use colored::Colorize;

use crate::{BranchName, CliOutput};

pub fn stack_adopt(parent_name: BranchName, _cli: &CliOutput) -> Result<()> {
    let graph = sage_graph::SageGraph::load_or_default()?;

    let current_branch = sage_git::branch::get_current()?;

    // Quick exit check if the parent and the current_branch are the same
    if parent_name == current_branch {
        bail!("Cannot set the parent to the same branch".red())
    }

    // Check to see if the parent branch exists, and is part of a stack
    if !graph.tracks(&parent_name) {
        // TODO: This could be incorrect, as there might be a loose branch with this name.
        bail!("Parent branch is not part of any stack".red())
    }

    // Early exit if the branch is already being tracked
    if graph.tracks(&current_branch) {
        // TODO: This could be wrong as we might be tracking it as a loose branch.
        bail!("This branch is already tracked".red())
    }

    Ok(())
}
