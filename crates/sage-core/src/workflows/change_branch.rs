use anyhow::{Result, anyhow, bail};
use sage_git::{
    branch::{exists, get_current, is_default, push, switch},
    helpers::{clean_untracked, reset_hard},
    repo::fetch_remote,
    stash::{self, stash_push},
    status::status,
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
}

pub fn change_branch(mut opts: ChangeBranchOpts, tui: &Tui) -> Result<()> {
    let current = get_current()?;
    let mut graph = SageGraph::load_or_default()?;
    // Determine if we are in a stack
    let in_stack = graph.tracks(&current) && graph.stack_of(&current).is_some();

    // Check if there are loose changes
    validate_status(tui)?;

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
        tui.hint("next steps:")?;
        tui.hint("  • make your changes")?;
        tui.hint("  • sg save - commit your work")?;
        tui.hint("  • sg stack - view your branches")?;
    }

    Ok(())
}

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

fn handle_fetch(tui: &Tui) -> Result<()> {
    let progress = tui.progress("fetching remote");
    fetch_remote()?;
    progress.done();

    Ok(())
}

fn validate_status(tui: &Tui) -> Result<()> {
    let progress = tui.progress("checking status");
    let current_status = status()?;
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

    Ok(())
}
