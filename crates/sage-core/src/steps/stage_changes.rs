use anyhow::Result;
use sage_fmt::MessageType;

pub fn stage_changes(repo: &sage_git::Repo, console: &sage_fmt::Console) -> Result<()> {
    let untracked_files = repo.untracked_files()?;
    let unstaged_files = repo.unstaged_files()?;
    let staged_changes = repo.staged_changes()?;

    let has_staged = !staged_changes.is_empty();
    let has_untracked = !untracked_files.is_empty();
    let has_unstaged = !unstaged_files.is_empty();

    if !has_untracked && !has_unstaged && !has_staged {
        console.message(MessageType::Info, "No changes")?;
        return Ok(());
    }

    if has_staged {
        console.message(MessageType::Success, "Using staged changes")?;
        return Ok(());
    }

    repo.stage_all()?;

    console.message(MessageType::Success, "Staged all changes")?;

    Ok(())
}
