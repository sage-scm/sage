use anyhow::{Context, Result, bail};
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

    let message = match message {
        Some(msg) => msg,
        None => bail!("Commit message required. Use --message or --ai."),
    };
    repo.create_commit(&message, false, false)?;

    if push {
        repo.push(force)?;
    }

    Ok(())
}
