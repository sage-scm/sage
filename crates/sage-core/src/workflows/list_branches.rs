use anyhow::Result;
use colored::{Color, Colorize};
use std::collections::HashMap;

pub fn list_branches(show_stack: bool) -> Result<()> {
    let repo = sage_git::Repo::open()?;
    let graph = sage_graph::SageGraph::load(&repo)?;
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
        .chain(remote_branches)
        .collect::<Vec<String>>();

    combined.sort();

    let mut stack_colors: HashMap<String, Color> = HashMap::new();

    if show_stack {
        const STACK_COLOR_PALETTE: [Color; 8] = [
            Color::BrightBlue,
            Color::BrightMagenta,
            Color::BrightCyan,
            Color::BrightYellow,
            Color::BrightGreen,
            Color::BrightRed,
            Color::Blue,
            Color::Magenta,
        ];
        let mut next_color_index = 0;

        for branch in &combined {
            if branch.ends_with('*') || branch.ends_with("HEAD") {
                continue;
            }

            let cleaned_name = repo.remove_ref(branch).replace("origin/", "");
            if let Some(stack_name) = graph.stack_name_for_branch(&cleaned_name) {
                stack_colors.entry(stack_name.clone()).or_insert_with(|| {
                    let color = STACK_COLOR_PALETTE[next_color_index % STACK_COLOR_PALETTE.len()];
                    next_color_index += 1;
                    color
                });
            }
        }
    }

    println!("Branches:");

    for branch in &combined {
        let cleaned_name = repo.remove_ref(branch).replace("origin/", "");
        if branch.ends_with('*') || branch.ends_with("HEAD") {
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

        if show_stack && let Some(stack_name) = graph.stack_name_for_branch(&cleaned_name) {
            if let Some(color) = stack_colors.get(stack_name.as_str()) {
                print!(" {}", format!("({stack_name})").color(*color));
            } else {
                print!(" {}", format!("({stack_name})").dimmed());
            }
        }

        let (above, below) = repo.above_below(branch)?;

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
