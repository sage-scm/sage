use anyhow::{Context, Result};
use colored::Colorize;
use sage_ai::commit_message;
use sage_fmt::MessageType;

use crate::{fetch_if_stale, stage_changes};

pub async fn save(
    mut message: Option<String>,
    force: bool,
    ai: bool,
    push: bool,
    console: &sage_fmt::Console,
) -> Result<()> {
    let repo = sage_git::Repo::open()?;
    let _ = fetch_if_stale(&repo, console)?;
    let _current_branch = repo.get_current_branch()?;

    stage_changes(&repo, console)?;

    if message.is_some() {
        console.message(MessageType::Info, "Using provided message")?;
    }

    if message.is_none() && ai {
        let progress = console.progress("Generating message with AI");
        let diff = repo.diff_ai()?;
        let generated = commit_message(&diff)
            .await
            .context("AI failed to generate a commit message")?;
        progress.done();
        console.message(MessageType::Success, "AI provided commit message")?;
        message = Some(generated);
    }

    repo.create_commit(&message.unwrap(), false, false)?;

    let last_commit = repo.get_current_commit()?;
    let mut hash = last_commit.to_hex().to_string();
    hash.truncate(8);
    console.message(
        MessageType::Success,
        &format!("Created commit {}", hash.dimmed()),
    )?;

    if push {
        repo.push(force)?;
    }

    Ok(())
}
