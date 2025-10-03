use anyhow::{Result, bail};

use crate::{fetch_if_stale, fuzzy_match_branch};

pub fn work(
    branch: String,
    parent: Option<String>,
    fuzzy: bool,
    push: bool,
    root: bool,
) -> Result<()> {
    let mut repo = sage_git::Repo::open()?;
    let _ = fetch_if_stale(&repo)?;
    let current_branch = repo.get_current_branch()?;

    if branch == current_branch {
        println!("Already on that branch");
        return Ok(());
    }

    if repo.has_branch(branch.to_string())? {
        repo.switch_branch(&branch)?;
        return Ok(());
    }

    if fuzzy {
        let branch_list = repo.list_branches()?;
        let potential_branch = fuzzy_match_branch(&branch, branch_list)?;
        match potential_branch {
            Some(branch) => {
                return repo.switch_branch(&branch);
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
    repo.switch_branch(&branch)?;

    if push {
        return repo.set_upstream();
    }

    Ok(())
}
