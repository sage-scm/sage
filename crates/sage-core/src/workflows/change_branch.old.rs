use anyhow::{Result, bail};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use sage_events::EventData;
use sage_git::{
    branch::{exists, get_current, get_default_branch, list_branches, push, switch},
    repo::{fetch_remote, get_commiter, get_repo_root},
};
use sage_graph::SageGraph;
use sage_tui::{MessageType, Tui};
use std::path::Path;

use crate::{BranchName, events::EventManager};

#[derive(Debug, Default)]
pub struct ChangeBranchOpts {
    /// Name of the branch
    pub name: BranchName,
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
    /// JSON output mode
    pub json_mode: bool,
}

pub fn change_branch(mut opts: ChangeBranchOpts, tui: &Tui) -> Result<()> {
    let current_branch = get_current()?;
    let mut graph = SageGraph::load_or_default()?;
    // determine if we are part of a stack
    let in_stack = graph.tracks(&current_branch) && graph.stack_of(&current_branch).is_some();

    // Start a fuzzy search
    if opts.fuzzy && !opts.name.is_empty() {
        match fuzzy_find_branch(&opts) {
            Ok(Some(branch)) => opts.name = BranchName::new(branch)?,
            Ok(None) => bail!("No branch found"),
            Err(e) => return Err(e),
        }
    } else if opts.name.is_empty() {
        // We need a name for the fuzzy search
        let branch_options = list_branches()?;
        // let branch = select("Switch to branch".into(), branch_options.local)?;
        let branch = branch_options.local.first().unwrap();
        opts.name = BranchName::new(branch)?;
    }

    // The final version of the name. After both fuzzy, and tui
    let name = opts.name;

    // Fetch remote if we need to
    if opts.fetch && !opts.json_mode {
        let progress = tui.progress("Fetching remote");
        fetch_remote()?;
        progress.done();
    } else if opts.fetch {
        fetch_remote()?;
    }

    // if the branch name and the current branch are the same name, we early exit.
    if name == current_branch {
        if !opts.json_mode {
            tui.message(MessageType::Warning, &format!("Already on {name}"))?;
        }
        return Ok(());
    }

    // Early exit if the user is switching to the default branch
    if name == get_default_branch()? {
        if !opts.json_mode {
            let progress = tui.progress("Switching branch");
            switch(&name, false)?;
            progress.finish(&format!("Switched to {name} 🚀"));
        } else {
            switch(&name, false)?;
        }

        // Track the branch switch event
        if let Ok(repo_root) = get_repo_root()
            && let Ok(event_manager) = EventManager::new(Path::new(&repo_root))
        {
            let _ = event_manager.track(EventData::BranchSwitched {
                from: current_branch.clone(),
                to: name.to_string(),
            });
        }

        return Ok(());
    }

    // Early exit if the branch name exists in another stack, and we are asked to create it.
    if opts.create && in_stack && graph.stack_of(&name).is_some() {
        if !opts.json_mode {
            tui.message(
                MessageType::Error,
                "Cannot create new branch: Branch already exists in a stack",
            )?;
        }
        return Ok(());
    }

    // Early exit if the branch already exists
    if exists(&name)? {
        if !opts.json_mode {
            tui.message(MessageType::Warning, "Branch exists - checking it out")?;
        }
        switch(&name, false)?;

        // Track the branch switch event
        if let Ok(repo_root) = get_repo_root()
            && let Ok(event_manager) = EventManager::new(Path::new(&repo_root))
        {
            let _ = event_manager.track(EventData::BranchSwitched {
                from: current_branch.clone(),
                to: name.to_string(),
            });
        }

        return Ok(());
    }

    // If the user wants this branch based off the root, we will switch to it first.
    if opts.use_root {
        if !opts.json_mode {
            let progress = tui.progress("Switching to root");
            switch(&get_default_branch()?, false)?;
            progress.finish(&format!("Switched to root ({})", get_default_branch()?));
        } else {
            switch(&get_default_branch()?, false)?;
        }
    }

    // Early exit if the parent branch does not exist.
    if !opts.parent.is_empty() && !exists(&opts.parent)? {
        if !opts.json_mode {
            tui.message(
                MessageType::Error,
                &format!("Failed to switch to parent: '{}' does not exist", opts.parent),
            )?;
        }
        return Ok(());
    }

    // If the user has provided a parent branch for us to use, we will switch to it frist.
    if !opts.parent.is_empty() {
        if !opts.json_mode {
            let progress = tui.progress("Switching to parent");
            switch(&opts.parent, false)?;
            progress.finish(&format!("Switched to parent ({})", opts.parent));
        } else {
            switch(&opts.parent, false)?;
        }
    }

    // Early exit if we are not allowed to create the branch, as there is nothing we can do.
    if !opts.create {
        return Ok(());
    }

    // Creating the branch
    if !opts.json_mode {
        let progress = tui.progress("Creating new branch");
        switch(&name, true)?;
        progress.finish(&format!("Created new branch: {}", name));
    } else {
        switch(&name, true)?;
    }

    // Track the branch creation event
    if let Ok(repo_root) = get_repo_root()
        && let Ok(event_manager) = EventManager::new(Path::new(&repo_root))
    {
        let from_branch = if !opts.parent.is_empty() {
            opts.parent.clone()
        } else if opts.use_root {
            get_default_branch()?
        } else {
            current_branch.clone()
        };

        let commit_id = if let Ok(output) = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
        {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            String::new()
        };

        let _ = event_manager.track(EventData::BranchCreated {
            name: name.to_string(),
            from_branch,
            commit_id,
        });
    }

    let (username, _) = get_commiter()?;
    let parent = if !opts.parent.is_empty() {
        &opts.parent
    } else {
        &current_branch
    };

    if in_stack {
        // We will add the branch to the stack.
        if let Some(stack_name) = graph.stack_name_of(parent).cloned() {
            graph.add_stack_child(
                &stack_name,
                parent,
                name.clone().into(),
                Some(username.clone()),
            )?;
        }
    }

    if !in_stack && opts.track {
        graph.add_loose_branch(name.clone().into(), parent.clone(), username.clone())?;
    }

    // Saving the updated graph (even if it didn't change)
    graph.save()?;

    // Push the branch if required
    if opts.push {
        if !opts.json_mode {
            let progress = tui.progress("Pushing to remote");
            push(&name, false)?;
            progress.done();
        } else {
            push(&name, false)?;
        }
    }

    if opts.announce && !opts.json_mode {
        tui.message(MessageType::Success, &format!("Switched to {} 🚀", name))?;
    }

    Ok(())
}

fn fuzzy_find_branch(opts: &ChangeBranchOpts) -> Result<Option<String>> {
    let branches = list_branches()?;
    let matcher = SkimMatcherV2::default();

    let mut best_match = None;
    let mut best_score = 0;

    // First check for exact match (case-insensitive)
    for branch in &branches.branches {
        if branch.eq_ignore_ascii_case(&opts.name) {
            return Ok(Some(branch.clone()));
        }
    }

    // If no exact match, perform fuzzy search
    for branch in &branches.branches {
        if let Some(score) = matcher.fuzzy_match(branch, &opts.name)
            && score > best_score
        {
            best_score = score;
            best_match = Some(branch.clone());
        }
    }

    // Use the best match if found
    if let Some(branch_name) = best_match {
        return Ok(Some(branch_name));
    }

    Ok(None)
}
