use crate::{CliOutput, ColorizeExt};
use anyhow::Result;
use colored::Colorize;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_default: bool,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Serialize)]
pub struct BranchListOutput {
    pub current_branch: String,
    pub default_branch: String,
    pub branches: Vec<BranchInfo>,
}

pub fn list_branches(relative: bool) -> Result<()> {
    list_branches_with_output(relative, None)
}

pub fn list_branches_with_output(relative: bool, cli: Option<&CliOutput>) -> Result<()> {
    let current_branch = sage_git::branch::get_current()?;
    let default_branch = sage_git::branch::get_default_branch()?;
    let branches = sage_git::branch::list_branches()?;

    let compare_branch = if relative {
        &current_branch
    } else {
        &default_branch
    };

    let mut branch_infos = Vec::new();

    for branch in branches.branches {
        let is_current = branch == current_branch;
        let is_default = branch == default_branch;

        // Get ahead/behind relative to default branch
        let (ahead, behind) = if is_default {
            (0, 0)
        } else {
            sage_git::branch::ahead_behind(&compare_branch, &branch)?
        };

        branch_infos.push(BranchInfo {
            name: branch.clone(),
            is_current,
            is_default,
            ahead: ahead.max(0) as usize,
            behind: behind.max(0) as usize,
        });
    }

    // If CLI output is provided and JSON mode is enabled, output JSON
    if let Some(cli) = cli {
        let output = BranchListOutput {
            current_branch: current_branch.clone(),
            default_branch: default_branch.clone(),
            branches: branch_infos.clone(),
        };
        cli.json_output(&output)?;

        // If JSON mode, don't print the formatted output
        if cli.is_json_mode() {
            return Ok(());
        }
    }

    // Print formatted output for non-JSON mode
    println!("{}", "Branches:".sage().bold());

    for branch_info in branch_infos {
        // Format branch indicator and name
        if branch_info.is_current {
            print!("{} ", "●".sage());
            print!("{}", branch_info.name.bright_yellow().bold());
        } else if branch_info.is_default {
            print!("  ");
            print!("{}", branch_info.name.bright_green());
        } else {
            print!("  ");
            print!("{}", branch_info.name.white());
        }

        // Add ahead/behind indicators
        if branch_info.ahead > 0 || branch_info.behind > 0 {
            print!(" ");
            if branch_info.ahead > 0 {
                print!("{}", format!("↑{}", branch_info.ahead).bright_green());
            }
            if branch_info.ahead > 0 && branch_info.behind > 0 {
                print!(" ");
            }
            if branch_info.behind > 0 {
                print!("{}", format!("↓{}", branch_info.behind).bright_red());
            }
        }

        // Add special labels
        if branch_info.is_current {
            print!(" {}", "(current)".gray());
        }
        if branch_info.is_default {
            print!(" {}", "(default)".gray());
        }

        println!();
    }

    Ok(())
}
