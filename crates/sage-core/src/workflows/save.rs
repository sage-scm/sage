use anyhow::{Result, bail};
use sage_config::Config;
use sage_git::branch::{get_current, push, stage_paths};
use sage_git::{
    branch::{changed_files, is_clean, stage_all, unstage_all},
    commit::{commit, commit_empty_with_output},
    status::{has_changes, has_staged_changes, has_unstaged_changes, has_untracked_files, status},
};
use sage_tui::{FileChange, MessageType, SummaryItem, Tui};

use crate::commit::commit_message;

#[derive(Debug, Default)]
pub struct SaveOpts {
    /// The message to commit with
    pub message: String,
    /// Paths to commit
    pub paths: Vec<String>,
    /// Commit all changes
    pub all: bool,
    /// Use AI to generate a commit message
    pub ai: bool,
    /// Amend the previous commit
    pub amend: bool,
    /// Create an empty commit
    pub empty: bool,

    pub push: bool,
    pub json_mode: bool,
}

pub async fn save(opts: &SaveOpts, tui: &Tui) -> Result<()> {
    let config = sage_config::ConfigManager::new()?;
    let cfg = config.load()?;

    // Early exit if we can
    early_exit(&opts)?;

    stage_files(&opts, &tui, &cfg)?;

    // Run hooks before using ai to generate
    run_hooks(&tui)?;

    let commit_message = get_commit_message(&opts, &tui).await?;

    if opts.empty && !opts.amend {
        let progress = tui.progress("Creating commit");
        commit_empty_with_output()?;
        progress.done();

        tui.message(MessageType::Success, "Created empty commit")?;
    }

    if !opts.empty && !opts.amend {
        let progress = tui.progress("Creating commit");
        commit(&commit_message)?;
        progress.done();

        tui.message(MessageType::Success, "Created commit")?;
    }

    push_to_remote(&opts, &tui)?;

    Ok(())
}

// Early exit on the follow scenarios
// - no commit message and not a valid reason
// - nothing to commit, and not amending, or creating an empty commit
fn early_exit(opts: &SaveOpts) -> Result<()> {
    if !opts.empty && !opts.amend && opts.message.is_empty() && !opts.ai {
        bail!(
            "Commit message is required. Provide a commit message as an argument, --ai to generate one, --empty for an empty commit, or --amend to amend the previous commit."
        )
    }

    if is_clean()? && !opts.empty && !opts.amend {
        bail!("Nothing to commit")
    }

    Ok(())
}

fn stage_files(opts: &SaveOpts, tui: &Tui, cfg: &Config) -> Result<()> {
    let status = status()?;
    let staged_changes = has_staged_changes()?;
    let unstaged_changes = has_unstaged_changes()?;
    let untracked_files = has_untracked_files()?;
    let mixed_staging = staged_changes && (unstaged_changes || untracked_files);

    // Early exit if empty
    if !(opts.empty || opts.amend) && status.is_clean() {
        bail!("nothing to commit")
    }

    // If paths are provided, stage them
    if !opts.paths.is_empty() {
        stage_paths(&opts.paths)?;
        return Ok(());
    }

    // Already has staged changes
    if staged_changes && !unstaged_changes && !untracked_files {
        return Ok(());
    }

    // Used asked to not be bothered.
    if mixed_staging && !cfg.save.ask_on_mixed_staging {
        return Ok(());
    }


    if changed_files()?.len() == 1 {
        // we will stage all the files, as there is only one. 
        stage_all()?;
        return Ok(());
    }

    // Creating the output for the user.
    list_changed_files(&tui)?;

    let action = tui.prompt("Stage files", &['a', 'e', 'c'])?;

    match action {
        'a' => {
            stage_all()?;
        }
        'e' => {
            let changes = changed_files()?
                .iter()
                .map(|c| format!("# {0}", c.name.clone()))
                .collect::<Vec<_>>();
            let result = tui
                .edit_text("Select files to stage")
                .initial_content(changes.join("\n"))
                .comments(vec![
                    "Remove the '#' from the front of the file to stage it",
                    "All lines with a '#' prefix will be ignored.",
                ])
                .strip_comments(true)
                .run()?;

            let result = result
                .lines()
                .map(|l| l.trim_start_matches('\n'))
                .filter(|f| !f.is_empty())
                .collect::<Vec<_>>();
            // Staging requested files
            stage_paths(&result)?;
        }
        _ => bail!("cancelled"),
    }

    Ok(())
}

fn run_hooks(tui: &Tui) -> Result<()> {
    let progress = tui.progress("Running pre-commit hooks");
    match sage_git::hooks::run_pre_commit() {
        Ok(()) => {
            progress.done();
            tui.message(MessageType::Success, "hooks passed\n")?;
        }
        Err(e) => {
            progress.done();
            tui.message(MessageType::Error, &format!("hooks failed: {}", e))?;
        }
    }
    Ok(())
}

/// Get the commit message for the changes
async fn get_commit_message(opts: &SaveOpts, tui: &Tui) -> Result<String> {
    if opts.ai {
        let progress = tui.progress("Generating message");

        // Generate message with ai
        let message_result = commit_message().await?;
        // Split into title/body on first blank line; if none, treat entire message as title
        let (title, body_opt) = if let Some((t, b)) = message_result.split_once("\n\n") {
            let b = if b.trim().is_empty() { None } else { Some(b) };
            (t, b)
        } else {
            (message_result.trim_end(), None)
        };
        progress.done();
        tui.commit_message(title, body_opt)?;

        let res = tui.prompt("proceed?", &['y', 'n', 'e'])?;

        match res {
            'y' => return Ok(message_result),
            'e' => {
                let edit_result = tui
                    .edit_text("Modify commit message")
                    .initial_content(message_result)
                    .strip_comments(true)
                    .run()?;

                return Ok(edit_result);
            }
            _ => bail!("cancelled"),
        }
    }

    if !opts.message.is_empty() {
        return Ok(opts.message.clone());
    }

    Ok(String::new())
}

fn list_changed_files(tui: &Tui) -> Result<()> {
    let mut changes = vec![];
    for change in changed_files()? {
        if change.additions > 0 && change.deletions > 0 {
            changes.push(FileChange::new(change.name).modified(change.additions, change.deletions));
        } else if change.additions > 0 {
            changes.push(FileChange::new(change.name).added(change.additions));
        } else {
            changes.push(FileChange::new(change.name).deleted(change.deletions));
        }
    }

    tui.file_list(&changes)
}

// Push the commits to remote
fn push_to_remote(opts: &SaveOpts, tui: &Tui) -> Result<()> {
    if !opts.push {
        return Ok(());
    }

    let progress = tui.progress("Pushing");
    let branch = get_current()?;
    match push(&branch, false) {
        Ok(()) => {
            progress.done();
            tui.message(MessageType::Success, "Pushed to remote")?;
        }
        Err(_) => {
            progress.done();
            tui.message(MessageType::Error, "Failed to push to remote")?;
        }
    }

    Ok(())
}
// TODO: add amend mode

// COMMIT MESSAGE GENERATION:
// TODO: add support for amend
// TODO: add support for pre-provided

// TODO: add empty commit mode
// TODO: track the event
