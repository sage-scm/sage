use anyhow::Result;
use colored::Colorize;
use sage_core::{CliOutput, events::EventManager};
use sage_events::{EventId, undo::UndoOperation};
use sage_git::{
    branch::{get_current, switch},
    repo::get_repo_root,
};
use std::path::Path;

pub fn undo(id: Option<String>, global_config: &crate::GlobalConfig) -> Result<()> {
    let cli_config = sage_core::cli::GlobalConfig::new(global_config.json, global_config.no_color);
    let cli = CliOutput::new(cli_config);
    let repo_root = get_repo_root()?;
    let event_manager = EventManager::new(Path::new(&repo_root))?;

    if let Some(id_str) = id {
        // Undo specific event
        if let Ok(uuid) = id_str.parse::<uuid::Uuid>() {
            let event_id = EventId(uuid);

            cli.step_start("Finding event");
            match event_manager.undo_event(&event_id) {
                Ok(undo_op) => {
                    cli.step_success("Found event", Some(&event_id.to_string().dimmed()));

                    cli.step_start(&format!("Undo: {}", undo_op.describe()));
                    execute_undo_operation(undo_op)?;
                    cli.step_success("Undo complete", None);
                }
                Err(e) => {
                    cli.step_error("Cannot undo", &e.to_string().red());
                }
            }
        } else {
            cli.step_error("Invalid event ID", &id_str.red());
        }
    } else {
        // Undo last undoable event
        cli.step_start("Finding last undoable event");
        match event_manager.undo_last() {
            Ok(undo_op) => {
                cli.step_success("Found event", None);

                cli.step_start(&format!("Undo: {}", undo_op.describe()));
                execute_undo_operation(undo_op)?;
                cli.step_success("Undo complete", None);
            }
            Err(e) => {
                cli.step_error("Cannot undo", &e.to_string().red());
            }
        }
    }

    Ok(())
}

pub fn history(global_config: &crate::GlobalConfig) -> Result<()> {
    let cli_config = sage_core::cli::GlobalConfig::new(global_config.json, global_config.no_color);
    let _cli = CliOutput::new(cli_config);
    let repo_root = get_repo_root()?;
    let event_manager = EventManager::new(Path::new(&repo_root))?;

    let history = event_manager.get_undo_history(50)?;

    if history.is_empty() {
        println!("No events in history");
        return Ok(());
    }

    println!("Showing last {} events:", history.len());
    println!();

    for (event, can_undo) in history.iter().rev() {
        let id_full = event.id.0.to_string();
        let id_str = id_full.split('-').next().unwrap_or("");
        let timestamp = event.timestamp.format("%Y-%m-%d %H:%M:%S");

        let status = if *can_undo {
            "✓".green()
        } else {
            "✗".red()
        };

        let event_desc = match &event.data {
            sage_events::event::EventData::CommitCreated {
                message, branch, ..
            } => {
                let msg_preview = message.lines().next().unwrap_or("");
                format!("Commit on {}: {}", branch.yellow(), msg_preview)
            }
            sage_events::event::EventData::BranchCreated {
                name, from_branch, ..
            } => {
                format!(
                    "Create branch {} from {}",
                    name.yellow(),
                    from_branch.dimmed()
                )
            }
            sage_events::event::EventData::BranchSwitched { from, to } => {
                format!("Switch {} → {}", from.dimmed(), to.yellow())
            }
            sage_events::event::EventData::BranchDeleted { name, .. } => {
                format!("Delete branch {}", name.red())
            }
            sage_events::event::EventData::CommitAmended { branch, .. } => {
                format!("Amend commit on {}", branch.yellow())
            }
            sage_events::event::EventData::Push {
                branch,
                commits,
                force,
                ..
            } => {
                let force_str = if *force { " (force)" } else { "" };
                format!(
                    "Push {} ({} commits){}",
                    branch.yellow(),
                    commits.len(),
                    force_str.red()
                )
            }
            _ => format!("{:?}", event.data.event_type()),
        };

        println!(
            "{} {} {} {}",
            status,
            id_str.dimmed(),
            timestamp.to_string().dimmed(),
            event_desc
        );

        if *can_undo {
            let explanation = event_manager.explain_undo(event);
            println!("    {}", explanation.dimmed());
        }
    }

    println!();
    println!();
    println!("Use 'sage undo <id>' to undo a specific event");

    Ok(())
}

fn execute_undo_operation(op: UndoOperation) -> Result<()> {
    use std::process::Command;

    match op {
        UndoOperation::ResetBranch {
            branch,
            to_commit,
            mode,
        } => {
            // First switch to the branch if needed
            let current = get_current()?;
            if current != branch {
                switch(&branch, false)?;
            }

            let mode_arg = match mode {
                sage_events::event::ResetMode::Soft => "--soft",
                sage_events::event::ResetMode::Mixed => "--mixed",
                sage_events::event::ResetMode::Hard => "--hard",
            };

            Command::new("git")
                .args(["reset", mode_arg, &to_commit])
                .output()?;
        }
        UndoOperation::CreateBranch { name, at_commit } => {
            Command::new("git")
                .args(["branch", &name, &at_commit])
                .output()?;
        }
        UndoOperation::DeleteBranch { name } => {
            Command::new("git").args(["branch", "-D", &name]).output()?;
        }
        UndoOperation::SwitchBranch { to_branch } => {
            switch(&to_branch, false)?;
        }
        UndoOperation::RenameBranch { from, to } => {
            Command::new("git")
                .args(["branch", "-m", &from, &to])
                .output()?;
        }
        UndoOperation::DropStash { stash_id } => {
            Command::new("git")
                .args(["stash", "drop", &stash_id])
                .output()?;
        }
        UndoOperation::CherryPick {
            commit,
            onto_branch,
        } => {
            switch(&onto_branch, false)?;

            Command::new("git")
                .args(["cherry-pick", &commit])
                .output()?;
        }
        UndoOperation::RevertCommit { commit } => {
            Command::new("git")
                .args(["revert", &commit, "--no-edit"])
                .output()?;
        }
    }

    Ok(())
}
