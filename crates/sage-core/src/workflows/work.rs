use anyhow::{Result, bail};
use colored::Colorize;
use sage_fmt::MessageType;

use crate::{fetch_if_stale, fuzzy_match_branch};

pub fn work(
    branch: String,
    parent: Option<String>,
    fuzzy: bool,
    push: bool,
    root: bool,
    console: &sage_fmt::Console,
) -> Result<()> {
    let mut repo = sage_git::Repo::open()?;
    let _ = fetch_if_stale(&repo, console)?;
    let current_branch = repo.get_current_branch()?;

    if branch == current_branch {
        console.message(MessageType::Info, "Already on the current branch")?;
        return Ok(());
    }

    if repo.has_branch(branch.to_string())? {
        repo.switch_branch(&branch)?;
        console.message(
            MessageType::Success,
            &format!("Switched to '{}'", branch.bright_blue()),
        )?;
        return Ok(());
    }

    if fuzzy {
        let branch_list = repo.list_branches()?;
        let potential_branch = fuzzy_match_branch(&branch, branch_list)?;
        match potential_branch {
            Some(branch) => {
                repo.switch_branch(&branch)?;
                console.message(
                    MessageType::Success,
                    &format!("Switched to '{}'", branch.bright_blue()),
                )?;
                return Ok(());
            }
            None => {
                bail!("No local branch found");
            }
        }
    }

    if root {
        let default_branch = repo.get_default_branch()?.replace("origin/", "");
        repo.switch_branch(&default_branch)?;
        println!("Switched to default branch");
    }

    if let Some(parent) = parent {
        if !repo.has_branch(parent.to_string())? {
            bail!("Parent branch not found");
        }
        repo.switch_branch(&parent)?;
    }

    repo.create_branch(&branch)?;
    console.message(MessageType::Success, "Created branch")?;
    repo.switch_branch(&branch)?;
    console.message(
        MessageType::Success,
        &format!("Switched to '{}'", branch.bright_blue()),
    )?;

    if push {
        repo.set_upstream()?;
        console.message(MessageType::Success, "Set upstream tracking")?;
    }

    Ok(())
}
