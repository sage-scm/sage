use anyhow::{Result, bail};
use colored::Colorize;

use crate::{BranchName, CliOutput};

use super::change_branch;

pub fn stack_init(stack_name: BranchName, cli: &CliOutput) -> Result<()> {
    let mut graph = sage_graph::SageGraph::load_or_default()?;

    let current_branch = sage_git::branch::get_current()?;

    // Quick exit check for a stack with the same name.
    if graph.tracks(&stack_name) {
        bail!("Stack or Branch with this name already exists".red())
    }

    let (name, _) = sage_git::repo::get_commiter()?;

    // TODO: We either want to discourage building a stack on a loose branch, or ask if they want
    // to proceed.
    // Check if the current branch is tracked already.
    if !graph.tracks(&current_branch) {
        // It does not track this branch.
        cli.step_start("Track current branch");
        graph.add_loose_branch(
            current_branch.clone(),
            sage_git::branch::get_default_branch()?,
            &name,
        )?;
        cli.step_success("Tracking current branch", Some(&current_branch.dimmed()));
    }

    // We will now create the new branch
    change_branch(
        super::ChangeBranchOpts {
            name: stack_name.clone(),
            parent: current_branch.clone(),
            create: true,
            fetch: true,
            use_root: false,
            push: false,
            fuzzy: false,
            track: false,
            announce: false,
        },
        cli,
    )?;

    // We will now init the stack
    cli.step_start("Initialising stack");
    graph.new_stack(stack_name.to_string(), stack_name.clone().into())?;
    cli.step_success("Stack initialized", Some(stack_name.as_str()));

    graph.save()?;

    Ok(())
}
