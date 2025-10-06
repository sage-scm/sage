use anyhow::Result;
use colored::Colorize;
use sage_fmt::MessageType;

use crate::{commit_message, fetch_if_stale, stage_changes};

pub async fn save(
    message: Option<String>,
    force: bool,
    ai: bool,
    push: bool,
    empty: bool,
    amend: bool,
    console: &sage_fmt::Console,
) -> Result<()> {
    let repo = sage_git::Repo::open()?;
    let _ = fetch_if_stale(&repo, console)?;
    let _current_branch = repo.get_current_branch()?;

    stage_changes(&repo, console)?;

    let msg = commit_message(&repo, console, message, ai).await?;

    repo.create_commit(&msg, empty, amend)?;

    let last_commit = repo.get_current_commit()?;
    let mut hash = last_commit.to_hex().to_string();
    hash.truncate(8);

    console.message(
        MessageType::Success,
        &format!("Created commit {}", hash.dimmed()),
    )?;

    if push {
        repo.push(force)?;
        console.message(MessageType::Success, "Pushed to remote")?;
    }

    Ok(())
}
