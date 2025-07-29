use crate::ColorizeExt;
use anyhow::Result;
use colored::Colorize;
use sage_git::{branch::get_current, commit::commits};

pub fn log_commits() -> Result<()> {
    let mut logs = commits()?;

    let current_branch = get_current()?;

    // Reverse the commits so that the latest commits are at the bottom
    logs.reverse();

    println!(
        "{} {}",
        "Branch History:".sage().bold(),
        current_branch.yellow()
    );
    if logs.is_empty() {
        println!("{}", "No commits found".bright_green());
        return Ok(());
    }

    // Group commits by date
    let mut current_date = String::new();

    for commit in &logs {
        // If we encounter a new date, print it
        if commit.date != current_date {
            current_date = commit.date.clone();
            println!();
            println!("{} {}", "Date:".bright_blue(), current_date.bold());
        }

        // Print commit info in the desired format
        println!(
            " {} {} {} @{}",
            "●".sage(),
            commit.hash.bright_yellow(),
            "by".gray(),
            commit.author
        );

        // Print the commit message indented
        if !commit.message.is_empty() {
            println!("   {}", commit.message.gray());
        }
    }

    Ok(())
}
