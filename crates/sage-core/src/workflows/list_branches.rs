use anyhow::Result;
use colored::Colorize;

use sage_git::Repo;

pub fn list_branches() -> Result<()> {
    let repo = sage_git::Repo::open()?;
    let current_branch = repo.get_current_branch()?;
    let default_branch = repo.get_default_branch()?.replace("origin/", "");

    let branches = repo.list_branches()?;
    let remote_branches = repo
        .list_remote_branches()?
        .into_iter()
        .filter(|x| !x.contains("HEAD") && !x.contains(&current_branch))
        .collect::<Vec<String>>();

    let mut combined = branches
        .into_iter()
        .chain(remote_branches.into_iter())
        .collect::<Vec<String>>();

    combined.sort();

    println!("Branches:");

    for branch in combined {
        let cleaned_name = repo.remove_ref(&branch).replace("origin/", "");
        if branch.ends_with("*") || branch.ends_with("HEAD") {
            continue;
        }

        // Print the branch name
        if cleaned_name == current_branch {
            print!(
                " {} {}",
                "●".bright_green(),
                cleaned_name.bold().bright_yellow()
            );
            print!(" {}", "(current)".dimmed());
        } else {
            print!("   {}", cleaned_name);
        }

        // Print markers
        if cleaned_name == default_branch {
            print!(" {}", "(default)".dimmed());
        }

        let (above, below) = repo.above_below(&branch)?;

        if above >= 1 {
            print!("{}", format!(" ↑{above}").bright_green().bold());
        }

        if below >= 1 {
            print!("{}", format!(" ↓{below}").bright_red().bold());
        }

        println!();
    }
    Ok(())
}
