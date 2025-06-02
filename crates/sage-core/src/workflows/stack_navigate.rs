use anyhow::{bail, Result};
use sage_git::branch;
use sage_tui::basic::select;

use crate::CliOutput;

use super::change_branch;

pub fn navigate(up: bool, cli: &CliOutput) -> Result<()> {
    let graph = sage_graph::SageGraph::load_or_default()?;
    let current_branch = branch::get_current()?;
    let stack = graph.stack_of(&current_branch);

    if stack.is_none() {
        bail!(format!("'{current_branch}' is not part of any stack"))
    }

    // We know that if we are here, that the current branch is part of a stack
    let unwrapped_stack = stack.unwrap();
    let change_to = if up {
        match unwrapped_stack.info(&current_branch) {
            Some(info) => info.parent.clone(),
            None => bail!("Branch info not found"),
        }
    } else {
        let children_branches = unwrapped_stack.children(&current_branch);
        if children_branches.is_empty() {
            bail!(format!(
                "There are no children branches for '{current_branch}'"
            ))
        }
        if children_branches.len() == 1 {
            children_branches[0].clone()
        } else {
            select("Multiple branches found".into(), children_branches)?
        }
    };

    change_branch(
        super::ChangeBranchOpts {
            name: change_to.to_string(),
            parent: String::new(),
            create: false,
            fetch: false,
            use_root: false,
            push: false,
            fuzzy: false,
            track: false,
            announce: false,
        },
        cli,
    )?;

    Ok(())
}
