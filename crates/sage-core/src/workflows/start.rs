use anyhow::{Result, bail};

use crate::fetch_if_stale;

#[derive(Debug, Clone)]
pub struct StartOptions {
    pub parent: Option<String>,
    pub stack: Option<String>,
    pub name: String,
    pub root: bool,
}

pub fn start(options: &StartOptions, console: &sage_fmt::Console) -> Result<()> {
    let repo = sage_git::Repo::open()?;
    let mut graph = sage_graph::SageGraph::load(&repo)?;

    fetch_if_stale(&repo, console)?;

    let current_branch = repo.get_current_branch()?;
    let mut parent_branch = current_branch.clone();

    if options.root {
        parent_branch = repo.get_default_branch()?;
    }

    if let Some(parent) = options.parent.clone() {
        if !repo.has_branch(parent.to_string())? {
            bail!("Parent branch not found");
        }
        parent_branch = parent;
    }

    if let Some(stack) = options.stack.clone() {
        if graph.stack_for_branch(&stack).is_none() {
            bail!("Stack not found");
        }
        parent_branch = stack;
    }

    // We need to make the new branch first, so that we can set the upstream tracking
    repo.create_branch_from(&options.name, &parent_branch)?;
    graph.create_stack(
        &repo,
        options.name.clone().to_string(),
        options.name.clone().to_string(),
        parent_branch.clone().to_string(),
    )?;
    repo.switch_branch(&options.name)?;

    graph.save(&repo)?;

    Ok(())
}
