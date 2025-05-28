use anyhow::{bail, Result};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use octocrab::current;
use sage_git::{
    branch::{
        exists, get_current, get_default_branch, is_default, is_default_branch, list_branches,
        push, switch,
    },
    repo::{fetch_remote, get_commiter},
};
use sage_graph::SageGraph;

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

// TODO: Implement the use of `opts.parent`

pub fn change_branch(mut opts: ChangeBranchOpts, cli: &CliOutput) -> Result<()> {
    let current_branch = get_current()?;
    let default_branch = get_default_branch()?;

    // Handle fuzzy search if enabled
    if opts.fuzzy && !opts.name.is_empty() {
        match fuzzy_find_branch(&opts) {
            Ok(Some(branch)) => opts.name = branch,
            Ok(None) => bail!("No branch found"),
            Err(e) => bail!(e),
        }
    } else if opts.name.is_empty() {
        // We will ask the user which branch they want to switch to
        let branches = list_branches()?;
        let branch = sage_tui::basic::select(String::from("Switch to branch"), branches)?;
        opts.name = branch;
    }

    let name = &opts.name;

    if opts.fetch {
        cli.step_start("Fetching remote");
        fetch_remote()?;
        cli.step_success("Fetched remote", None);
    }

    if *name == current_branch {
        println!("⚠️  Already on {name}");
        return Ok(());
    }

    // Early exit if they are switching to the default branch.
    if &current_branch == &default_branch && &default_branch == name {
        cli.step_start("Switching branch");
        switch(name, false)?;
        cli.step_success_with_emoji(&format!("Switched to {name}"), None, "🚀");
        return Ok(());
    }

    if opts.use_root && default_branch == current_branch {
        cli.step_start("Switching branch");
        switch(&default_branch, false)?;
        cli.step_success("Switched to root branch", None);
    }

    // Let's see if the new brach already exists.
    if exists(name)? {
        cli.warning("Branch exists - checking it out.");
        // We will simply switch to the branch.
        switch(name, false)?;
    } else if opts.create {
        // We will create the branch.
        cli.step_start("Creating new branch");
        switch(name, true)?;
        cli.step_success(&format!("Created branch {name}"), None);
    }

    if opts.push {
        cli.step_start("Pushing to remote");
        push(name, false)?;
        cli.step_success(&format!("Pushed origin/{name}"), None);
    }

    if opts.track {
        let mut graph = SageGraph::load_or_default()?;
        if !graph.tracks(name) && !graph.is_loose(name) {
            let (user_name, _) = get_commiter()?;
            // TODO: fix. We don't want to track the current branch as the parent if the root is
            // the provided as the branch to use as the parent.
            let parent = if opts.use_root {
                default_branch
            } else {
                current_branch
            };
            graph.add_loose_branch(name.into(), parent, user_name)?;
            graph.save()?;
        }
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
    for branch in branches {
        if let Some(score) = matcher.fuzzy_match(&branch, &opts.name) {
            if score > best_score {
                best_score = score;
                best_match = Some(branch);
            }
        }
    }

    // Use the best match if found
    if let Some(branch_name) = best_match {
        println!("🔍 Fuzzy matched '{}' to '{}'", opts.name, branch_name);
        Ok(Some(branch_name))
    } else {
        Ok(None)
    }
}
