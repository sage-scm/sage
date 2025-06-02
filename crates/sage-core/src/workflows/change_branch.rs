use anyhow::{bail, Result};
use colored::Colorize;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use sage_git::{
    branch::{exists, get_current, get_default_branch, list_branches, push, switch},
    repo::{fetch_remote, get_commiter},
};
use sage_graph::SageGraph;
use sage_tui::basic::select;

use crate::CliOutput;

#[derive(Debug, Default)]
pub struct ChangeBranchOpts {
    /// Name of the branch
    pub name: String,
    /// Parent branch of the new branch
    pub parent: String,
    /// Can the branch be created?
    pub create: bool,
    /// Should we fetch remote first?
    pub fetch: bool,
    /// Should we switch to root branch first?
    pub use_root: bool,
    /// Push to remote?
    pub push: bool,
    /// Use fuzzy search for branch name?
    pub fuzzy: bool,
    /// Track the branch automatically
    pub track: bool,
    /// Announce the branch change
    pub announce: bool,
}

pub fn change_branch(mut opts: ChangeBranchOpts, cli: &CliOutput) -> Result<()> {
    let current_branch = get_current()?;
    let mut graph = SageGraph::load_or_default()?;
    // determine if we are part of a stack
    let in_stack = graph.tracks(&current_branch) && graph.stack_of(&current_branch).is_some();

    // Start a fuzzy search
    if opts.fuzzy && !opts.name.is_empty() {
        match fuzzy_find_branch(&opts) {
            Ok(Some(branch)) => opts.name = branch,
            Ok(None) => bail!("No branch found"),
            Err(e) => return Err(e),
        }
    } else if opts.name.is_empty() {
        // We need a name for the fuzzy search
        let branches = list_branches()?;
        let branch = select("Switch to branch".into(), branches)?;
        opts.name = branch;
    }

    // The final version of the name. After both fuzzy, and tui
    let name = opts.name;

    // Fetch remote if we need to
    if opts.fetch {
        cli.step_start("Fetching remote");
        fetch_remote()?;
        cli.step_success("Fetched remote", None);
    }

    // if the branch name and the current branch are the same name, we early exit.
    if name == current_branch {
        cli.warning(&format!("Already on {name}"));
        return Ok(());
    }

    // Early exit if the user is switching to the default branch
    if name == get_default_branch()? {
        cli.step_start("Switching branch");
        switch(&name, false)?;
        // TODO: Probably need to determine if they want us to announce
        cli.step_success_with_emoji(&format!("Switch to {name}"), None, "🚀");
        return Ok(());
    }

    // Early exit if the branch name exists in another stack, and we are asked to create it.
    if opts.create && in_stack && graph.stack_of(&name).is_some() {
        cli.step_error(
            "Cannot create new branch",
            &"Branch already exists to a stack".red(),
        );
        return Ok(());
    }

    // Early exit if the branch already exists
    if exists(&name)? {
        cli.warning("Branch exists - checking it out");
        switch(&name, false)?;
        return Ok(());
    }

    // If the user wants this branch based off the root, we will switch to it first.
    if opts.use_root {
        cli.step_start("Switching to root");
        switch(&get_default_branch()?, false)?;
        cli.step_success("Switched to root", Some(&get_default_branch()?.dimmed()));
    }

    // Early exit if the parent branch does not exist.
    if !opts.parent.is_empty() && !exists(&opts.parent)? {
        cli.step_error(
            "Failed to switch to parent",
            &format!("'{}' does not exist", opts.parent).red(),
        );
        return Ok(());
    }

    // If the user has provided a parent branch for us to use, we will switch to it frist.
    if !opts.parent.is_empty() {
        cli.step_start("Switching to parent");
        switch(&opts.parent, false)?;
        cli.step_success("Switched to parent", Some(&opts.parent.dimmed()));
    }

    // Early exit if we are not allowed to create the branch, as there is nothing we can do.
    if !opts.create {
        return Ok(());
    }

    // Creating the branch
    cli.step_start("Creating new branch");
    switch(&name, true)?;
    cli.step_success("Created new branch", Some(&name));

    if in_stack {
        // We will add the branch to the stack.
        let (username, _) = get_commiter()?;
        let parent = if !opts.parent.is_empty() {
            &opts.parent
        } else {
            &current_branch
        };
        if let Some(stack_name) = graph.stack_name_of(&parent).cloned() {
            graph.add_stack_child(&stack_name, &parent, name.clone(), Some(username))?;
        }
        graph.save()?;
    }

    // Push the branch if required
    if opts.push {
        let spinner = cli.spinner("Pushing to remote");
        push(&name, false)?;
        spinner.finish_success("Pushed to remote", None);
    }

    if opts.announce {
        cli.step_success_with_emoji(&format!("Switched to {name}"), None, "🚀");
    }

    Ok(())
}

fn fuzzy_find_branch(opts: &ChangeBranchOpts) -> Result<Option<String>> {
    let branches = list_branches()?;
    let matcher = SkimMatcherV2::default();

    let mut best_match = None;
    let mut best_score = 0;

    // First check for exact match (case-insensitive)
    for branch in &branches {
        if branch.eq_ignore_ascii_case(&opts.name) {
            return Ok(Some(branch.clone()));
        }
    }

    // If no exact match, perform fuzzy search
    for branch in &branches {
        if let Some(score) = matcher.fuzzy_match(&branch, &opts.name) {
            if score > best_score {
                best_score = score;
                best_match = Some(branch.clone());
            }
        }
    }

    // Use the best match if found
    if let Some(branch_name) = best_match {
        return Ok(Some(branch_name));
    }

    Ok(None)
}
