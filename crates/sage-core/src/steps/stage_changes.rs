use anyhow::Result;
use sage_fmt::MessageType;

pub fn stage_changes(
    repo: &sage_git::Repo,
    console: &sage_fmt::Console,
    paths: Option<Vec<String>>,
) -> Result<()> {
    if let Some(stage_paths) = paths {
        repo.stage_paths(stage_paths)?;
        console.message(MessageType::Success, "Staged provided paths")?;
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use sage_git::testing::TestRepo;

    #[test]
    fn preserves_partially_staged_files() -> anyhow::Result<()> {
        let repo = TestRepo::builder().with_initial_commit().build()?;

        repo.write("file.txt", "line 1\n")?;
        repo.commit_all("add file")?;

        repo.write("file.txt", "line 1\nline 2 staged\n")?;
        repo.run_git(["add", "file.txt"])?;
        repo.write("file.txt", "line 1\nline 2 staged\nline 3 unstaged\n")?;

        let console = sage_fmt::Console::new();
        stage_changes(repo.repo(), &console, None)?;

        let staged = repo.staged_changes()?;
        assert_eq!(staged, vec!["file.txt"]);

        let unstaged = repo.unstaged_files()?;
        assert_eq!(unstaged, vec!["file.txt"]);

        Ok(())
    }
}
