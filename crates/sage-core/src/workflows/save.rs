use anyhow::Result;
use sage_fmt::MessageType;

use crate::{fetch_if_stale, stage_changes};

pub fn save(
    message: Option<String>,
    force: bool,
    ai: bool,
    push: bool,
    console: &sage_fmt::Console,
) -> Result<()> {
    let mut repo = sage_git::Repo::open()?;
    let _ = fetch_if_stale(&repo, console)?;
    let _current_branch = repo.get_current_branch()?;

    stage_changes(&mut repo, &console)?;

    if message.is_some() {
        console.message(MessageType::Info, "Using provided message")?;
    }

    repo.create_commit(&message.unwrap(), false, false)?;

    if push {
        repo.push(force)?;
    }

    Ok(())
}
