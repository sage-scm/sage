use crate::ColorizeExt;
use anyhow::Result;
use colored::Colorize;

pub fn list_branches() -> Result<()> {
    let current_branch = sage_git::branch::get_current()?;
    let default_branch = sage_git::branch::get_default_branch()?;
    let branches = sage_git::branch::list_branches()?;

    println!("{}", "Branches:".sage().bold());
    println!();

    for branch in branches {
        let is_current = branch == current_branch;
        let is_default = branch == default_branch;

        // Get ahead/behind relative to default branch
        let (ahead, behind) = if is_default {
            (0, 0)
        } else {
            sage_git::branch::ahead_behind(&default_branch, &branch)?
        };

        // Format branch indicator and name
        if is_current {
            print!("{} ", "●".sage());
            print!("{}", branch.bright_yellow().bold());
        } else if is_default {
            print!("  ");
            print!("{}", branch.bright_green());
        } else {
            print!("  ");
            print!("{}", branch.white());
        }

        // Add ahead/behind indicators
        if ahead > 0 || behind > 0 {
            print!(" ");
            if ahead > 0 {
                print!("{}", format!("↑{}", ahead).bright_green());
            }
            if ahead > 0 && behind > 0 {
                print!(" ");
            }
            if behind > 0 {
                print!("{}", format!("↓{}", behind).bright_red());
            }
        }

        // Add special labels
        if is_current {
            print!(" {}", "(current)".gray());
        }
        if is_default {
            print!(" {}", "(default)".gray());
        }

        println!();
    }

    Ok(())
}
