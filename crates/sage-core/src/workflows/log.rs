use anyhow::Result;
use colored::Colorize;

pub fn log() -> Result<()> {
    let repo = sage_git::Repo::open()?;
    let current_branch = repo.get_current_branch()?;

    let mut logs = repo.get_commits()?;
    logs.reverse();

    println!(
        "{} {}",
        "Branch History:".bright_green().bold(),
        current_branch.yellow()
    );

    if logs.is_empty() {
        println!("{}", "No commits found".bright_red());
        return Ok(());
    }

    // Group commits by date
    let mut current_date = String::new();

    for commit in logs {
        // If we encounter a new date, print it
        if commit.date != current_date {
            current_date = commit.date.clone();
            println!();
            println!("{} {}", "Date:".bright_blue(), current_date.bold());
        }

        // Print commit info in the desired format
        println!(
            " {} {} {} @{}",
            "●".bright_green(),
            commit.hash.bright_yellow(),
            "by",
            commit.author
        );

        // Print he commit message indented
        if !commit.message.is_empty() {
            println!("  {}", commit.message);
        }
    }

    Ok(())
}
