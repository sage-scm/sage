use anyhow::{Result, bail};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use sage_git::{
    branch::{exists, get_current, is_default, pull, push, switch},
    helpers::{clean_untracked, reset_hard},
    repo::{fetch_remote, get_commiter},
    stash::stash_push,
    status::{GitStatus, status},
};
use sage_graph::SageGraph;
use sage_tui::{MessageType, SummaryItem, Tui};

use crate::BranchName;

#[derive(Debug, Default)]
pub struct ChangeBranchOpts {
    /// Name of the branch
    pub name: BranchName,
    /// Parent branch of the new branch
    pub parent: String,
    /// Can the branch be created
    pub create: bool,
    /// Fetch before creating the branch
    pub fetch: bool,
    /// Push to remote after creating branch
    pub push: bool,
    /// Track the branch automatically
    pub track: bool,
    /// Fuzzy find the branch
    pub fuzzy: bool,
}

pub fn change_branch(mut opts: ChangeBranchOpts, tui: &Tui) -> Result<()> {
    let mut graph = SageGraph::load_or_default()?;
    let status = status()?;

    // Early exit on invalid options
    early_exists(&opts, &status, &graph)?;

    // Check if there are loose changes
    validate_status(&status, tui)?;

    if opts.fuzzy && !opts.name.is_empty() {
        match fuzzy_find_branch(&opts.name) {
            Ok(Some(branch)) => opts.name = BranchName::new(branch)?,
            Ok(None) => bail!("No branch found"),
            Err(e) => return Err(e),
        }
    }

    if opts.fetch {
        handle_fetch(tui)?;
    }

    let branch_exists = exists(&opts.name).unwrap_or_default();
    let mut created = false;

    if is_default(&opts.name).unwrap_or_default() {
        let progress = tui.progress("switching to default branch");
        switch(&opts.name, false)?;
        progress.done();
        tui.message(MessageType::Success, "Switched to default branch")?;

        return Ok(());
    }

    if branch_exists {
        switch_existing(&opts.name, tui)?;
    } else {
        create_new_branch(&opts.name, tui)?;
        created = true;
    }

    if created {
        track_in_stack(&opts, tui, &mut graph)?;
        graph.save()?;
    }

    if created {
        tui.hint("next steps:")?;
        tui.hint("  • make your changes")?;
        tui.hint("  • sg save - commit your work")?;
        tui.hint("  • sg stack - view your branches")?;
    }

    Ok(())
}

/// Early exit the branch change process if one of these conditions are met:
/// - Parent branch does not exist
/// - Cannot switch on a detached head
/// - Cannot create a branch with a fuzzy name
/// - Branch exists within another stack
/// - Branch is not part of the current stack
fn early_exists(opts: &ChangeBranchOpts, status: &GitStatus, graph: &SageGraph) -> Result<()> {
    if !opts.parent.is_empty() && !exists(&opts.parent).unwrap_or_default() {
        bail!("Parent branch does not exist");
    }

    if status.is_detached_head() {
        bail!("Cannot switch on a detached head");
    }

    if opts.create && opts.fuzzy {
        bail!("Cannot create a branch with a fuzzy name");
    }

    let stack = graph.stack_of(&opts.name);

    if stack.is_some() && opts.create {
        bail!("Branch exists within another stack");
    }

    if stack.is_some() && !opts.create && !stack.unwrap().contains(&opts.name) {
        bail!("Branch is not part of the current stack");
    }

    if opts.name == get_current()? {
        bail!("Already on that branch")
    }

    Ok(())
}

/// Creates a new branch with the given name.
fn create_new_branch(name: &str, tui: &Tui) -> Result<()> {
    let progress = tui.progress("creating branch");
    switch(name, true)?;
    progress.done();

    tui.message(
        MessageType::Success,
        &format!("created and switched to '{}'", name),
    )?;

    Ok(())
}

/// Switches to an existing branch.
fn switch_existing(name: &str, tui: &Tui) -> Result<()> {
    let choice = tui.prompt(&format!("'{}' exists, switch to it", name), &['y', 'n'])?;

    if choice != 'y' {
        bail!("Cancelled");
    }

    let progress = tui.progress(&format!("switching to '{}'", name));
    switch(name, false)?;
    progress.done();

    tui.message(MessageType::Success, &format!("switched to '{}'", name))?;

    Ok(())
}

/// Fetches the latest changes from the remote.
fn handle_fetch(tui: &Tui) -> Result<()> {
    let progress = tui.progress("fetching remote");
    fetch_remote()?;
    progress.done();

    Ok(())
}

/// Validates the status of the current branch.
/// - Checks for uncommitted changes
/// - Checks for untracked files
/// - Checks for unpushed commits
fn validate_status(current_status: &GitStatus, tui: &Tui) -> Result<()> {
    let progress = tui.progress("checking status");
    progress.done();

    if current_status.is_dirty() {
        tui.summary(&[SummaryItem::Count(
            "uncommitted changes".into(),
            current_status.unstaged_files_count() + current_status.staged_files_count(),
        )])?;

        tui.message(MessageType::Warning, "uncommitted changes detected")?;
        let choice = tui.prompt("action", &['s', 'd', 'c'])?;

        match choice {
            's' => {
                let progress = tui.progress("stashing changes");
                stash_push(None)?;
                progress.done();

                tui.message(MessageType::Success, "changes stashed")?;
                tui.hint("restore with: git stash pop")?;
            }
            'd' => {
                let confirm = tui.prompt("discard all changes", &['y', 'n'])?;
                if confirm == 'y' {
                    let progress = tui.progress("discarding changes");
                    reset_hard("HEAD")?;
                    clean_untracked(true, true)?;
                    progress.done();

                    tui.message(MessageType::Success, "changes discarded")?;
                } else {
                    bail!("Cancelled");
                }
            }
            _ => bail!("Cancelled"),
        }
    }

    // Check if there are unpushed commits
    if current_status.needs_push() || current_status.needs_pull() {
        // TODO: Show the summary to the user of the above-below.

        if current_status.needs_pull() {
            let confirm = tui.prompt("pull changes?", &['y', 'n'])?;
            if confirm == 'y' {
                let progress = tui.progress("pulling changes");
                pull()?;
                progress.done();
            }
        }

        if current_status.needs_push() {
            let confirm = tui.prompt("push changes?", &['y', 'n'])?;
            if confirm == 'y' {
                let progress = tui.progress("pushing changes");
                push(&get_current()?, false)?;
                progress.done();
            }
        }
    }

    Ok(())
}

/// Fuzzy find a branch by name.
/// - Only branches that are not the current branch, and are local.
fn fuzzy_find_branch(name: &str) -> Result<Option<String>> {
    let branches = sage_git::branch::list_branches()?;
    let matcher = SkimMatcherV2::default();

    let mut best_match = None;
    let mut best_score = 0;

    // First check for exact match (case-insensitive)
    for branch in &branches.branches {
        if branch.eq_ignore_ascii_case(&name) {
            return Ok(Some(branch.clone()));
        }
    }

    // If no exact match, perform fuzzy search
    for branch in &branches.branches {
        if let Some(score) = matcher.fuzzy_match(branch, &name)
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

/// Track the branch in the stack
fn track_in_stack(opts: &ChangeBranchOpts, tui: &Tui, graph: &mut SageGraph) -> Result<()> {
    let (username, _) = get_commiter()?;

    // Get stack information and extract what we need before mutations
    let stack_name = graph.stack_of(&opts.name).map(|s| s.name.clone());
    let has_stack = stack_name.is_some();

    if !graph.tracks(&opts.parent) {
        let res = tui.prompt("Track parent?", &['y', 'n'])?;
        if res == 'y' {
            let progress = tui.progress("tracking parent");
            graph.add_loose_branch(
                opts.parent.to_string(),
                // Defaulting to default branch
                sage_git::branch::get_default_branch()?,
                username.clone(),
            )?;
            progress.done();

            tui.message(MessageType::Success, "Parent tracked")?;
        }
    }

    let progress = tui.progress("tracking branch");
    if !has_stack && opts.track {
        graph.add_loose_branch(opts.name.to_string(), opts.parent.clone(), username.clone())?;
    }

    if let Some(stack_name) = stack_name {
        if opts.track {
            graph.add_stack_child(
                &stack_name,
                &opts.parent,
                opts.name.to_string(),
                Some(username),
            )?;
        }
    }
    progress.done();

    tui.message(MessageType::Success, "Branch tracked")?;

    Ok(())
}

// TODO: Add event tracking to allow for history
