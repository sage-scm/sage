use anyhow::{Result, anyhow};
use sage_events::EventData;
use sage_git::{
    amend::{self, AmendOpts},
    branch::{get_current, is_clean, push, stage_all, unstage_all},
    commit::{commit_empty_with_output, commit_with_output},
    repo::get_repo_root,
    status::{has_changes, has_staged_changes, has_unstaged_changes, has_untracked_files},
};
use sage_tui::{MessageType, Tui};
use std::path::Path;

use crate::commit::commit_message;
use crate::events::EventManager;

#[derive(Debug, Default)]
pub struct SaveOpts {
    /// The message to commit with
    pub message: String,
    /// Commit all changes
    pub all: bool,
    /// Commit only these paths
    pub paths: Vec<String>,
    /// Use AI to generate a commit message
    pub ai: bool,
    /// Amend the previous commit
    pub amend: bool,
    /// Push to remote
    pub push: bool,
    /// Create an empty git commit
    pub empty: bool,
    /// JSON output mode
    pub json_mode: bool,
}

pub async fn save(opts: &SaveOpts, tui: &Tui) -> Result<()> {
    let config = sage_config::ConfigManager::new()?;
    let cfg = config.load()?;

    // Check if a message is required (do this early)
    if !opts.empty && !opts.amend && opts.message.is_empty() && !opts.ai {
        return Err(anyhow!(
            "Commit message is required. Use -m to provide a message, --ai to generate one, --empty for an empty commit, or --amend to amend the previous commit."
        ));
    }

    // Early exit if working tree is clean and we're not creating an empty commit
    if is_clean()? && !opts.empty && !opts.amend {
        if !opts.json_mode {
            tui.message(MessageType::Warning, "Working tree is clean")?;
        }
        return Ok(());
    }

    // Handle staging
    if !opts.json_mode {
        let mut progress = tui.progress("Staging files");
        stage_correct_files(opts, tui, &cfg, &mut progress)?;
        progress.done();
    } else {
        let mut progress = tui.progress("");
        stage_correct_files(opts, tui, &cfg, &mut progress)?;
    }
    let commit_message = get_commit_message(opts, tui).await?;

    if opts.empty && !opts.amend {
        let progress = if !opts.json_mode {
            tui.progress("Creating empty commit")
        } else {
            tui.progress("")
        };
        // Create the empty commit and get the commit ID
        let result = match commit_empty_with_output() {
            Ok(r) => r,
            Err(e) => {
                // Extract the actual error message without the prefix
                let error_msg = e.to_string();
                let clean_error = if error_msg.starts_with("Failed to create empty commit: ") {
                    error_msg
                        .trim_start_matches("Failed to create empty commit: ")
                        .to_string()
                } else {
                    error_msg
                };

                // Display a more helpful error message
                progress.fail(&format!("Failed to create empty commit: {}", clean_error));
                return Ok(());
            }
        };

        // Display any hook output
        if !opts.json_mode {
            if let Some(hook_output) = &result.hook_output {
                for line in hook_output.lines() {
                    tui.println(&format!("    {}", line), None)?;
                }
            }
        }

        progress.finish(&format!("Write empty commit ({})", result.commit_id));

        // Track the empty commit event
        if let Ok(repo_root) = get_repo_root()
            && let Ok(event_manager) = EventManager::new(Path::new(&repo_root))
        {
            let current_branch = get_current()?;
            let _ = event_manager.track(EventData::CommitCreated {
                commit_id: result.commit_id.clone(),
                message: "[empty commit]".to_string(),
                files_changed: vec![],
                branch: current_branch,
            });
        }

        push_changes(opts, tui)?;
        return Ok(());
    }

    if opts.amend {
        let amend_opts = AmendOpts {
            message: commit_message.clone(),
            empty: opts.empty,
            no_edit: (opts.empty && opts.message.is_empty())
                || (!opts.message.is_empty() && !has_staged_changes()?), // Use no_edit if we're keeping the previous message
        };
        let progress = if !opts.json_mode {
            tui.progress("Amending commit")
        } else {
            tui.progress("")
        };

        // Get the old commit ID before amending
        let old_commit_id = if let Ok(output) = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
        {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            String::new()
        };

        amend::amend(&amend_opts)?;

        // Get the new commit ID after amending
        let new_commit_id = if let Ok(output) = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
        {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            String::new()
        };

        progress.done();

        // Track the amend event
        if let Ok(repo_root) = get_repo_root()
            && let Ok(event_manager) = EventManager::new(Path::new(&repo_root))
        {
            let current_branch = get_current()?;
            let _ = event_manager.track(EventData::CommitAmended {
                old_commit: old_commit_id,
                new_commit: new_commit_id,
                branch: current_branch,
            });
        }

        push_changes(opts, tui)?;
        return Ok(());
    }

    let progress = if !opts.json_mode {
        tui.progress("Creating commit")
    } else {
        tui.progress("")
    };

    // Create the commit and get the commit ID
    let result = match commit_with_output(&commit_message) {
        Ok(r) => r,
        Err(e) => {
            // Extract the actual error message without the "Failed to commit: " prefix
            let error_msg = e.to_string();
            let clean_error = if error_msg.starts_with("Failed to create commit: ") {
                error_msg
                    .trim_start_matches("Failed to create commit: ")
                    .to_string()
            } else if error_msg.starts_with("Failed to commit: ") {
                error_msg
                    .trim_start_matches("Failed to commit: ")
                    .to_string()
            } else {
                error_msg
            };

            // Display a more helpful error message
            if clean_error.is_empty() {
                progress.fail("No changes to commit (working directory is clean)");
            } else {
                progress.fail(&format!("Failed to commit: {}", clean_error));
            }
            return Ok(());
        }
    };

    // Display any hook output
    if !opts.json_mode {
        if let Some(hook_output) = &result.hook_output {
            for line in hook_output.lines() {
                tui.println(&format!("    {}", line), None)?;
            }
        }
    }

    progress.finish(&format!("Write commit ({})", result.commit_id));

    // Track the commit event
    if let Ok(repo_root) = get_repo_root()
        && let Ok(event_manager) = EventManager::new(Path::new(&repo_root))
    {
        let current_branch = get_current()?;

        // Get list of changed files
        let files_changed = if let Ok(output) = std::process::Command::new("git")
            .args([
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "-r",
                &result.commit_id,
            ])
            .output()
        {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        };

        let _ = event_manager.track(EventData::CommitCreated {
            commit_id: result.commit_id.clone(),
            message: commit_message.clone(),
            files_changed,
            branch: current_branch,
        });
    }

    // Handle push if requested
    push_changes(opts, tui)?;

    Ok(())
}

/// Push changes to remote.
fn push_changes(opts: &SaveOpts, tui: &Tui) -> Result<()> {
    if !opts.push {
        return Ok(());
    }

    let progress = if !opts.json_mode {
        tui.progress("Pushing to remote")
    } else {
        tui.progress("")
    };

    let branch = get_current()?;
    push(&branch, false)?;

    progress.finish(&format!("Pushed to remote ({})", branch));
    Ok(())
}

/// Determines the files to stage for the commit.
fn stage_correct_files(
    opts: &SaveOpts,
    tui: &Tui,
    cfg: &sage_config::Config,
    progress: &mut sage_tui::ProgressHandle,
) -> Result<()> {
    let staged_changes = has_staged_changes()?;
    let unstaged_changes = has_unstaged_changes()?;
    let untracked_files = has_untracked_files()?;
    let changed_files = has_changes()?;

    // User wants everything staged no matter what.
    if opts.all {
        stage_all()?;
        progress.set_message("Staged all files".to_string());
        return Ok(());
    }

    // If the user provided files to commit.
    if !opts.paths.is_empty() {
        // TODO: Implement staging specific files properly
        // For now, just stage all changes
        stage_all()?;
        progress.set_message("Staged all files".to_string());
        return Ok(());
    }

    // User is amending the last commit without changes.
    if opts.amend && opts.empty {
        unstage_all()?;
        progress.set_message("Unstaged all files".to_string());
        return Ok(());
    }

    // No files changed, no need to stage.
    if opts.amend && !opts.message.is_empty() && !changed_files {
        progress.set_message("No files staged".to_string());
        return Ok(());
    }

    // User is amending last commit with changes.
    if opts.amend && !staged_changes {
        stage_all()?;
        progress.set_message("Staged all files".to_string());
        return Ok(());
    }

    // User only has unstaged / untracked changes.
    if !staged_changes && (unstaged_changes || untracked_files) {
        stage_all()?;
        progress.set_message("Staged all files".to_string());
        return Ok(());
    }

    // User has already staged their changes.
    if staged_changes && !unstaged_changes && !untracked_files {
        progress.set_message("Using staged changes".to_string());
        return Ok(());
    }

    // Early exit if there are staged changes and the ask_on_mixed_staging is disabled
    if staged_changes && !cfg.save.ask_on_mixed_staging {
        progress.set_message("Using staged changes".to_string());
        return Ok(());
    }

    // User has both staged and unstaged/untracked changes.
    if staged_changes && (unstaged_changes || untracked_files) {
        if !opts.json_mode {
            tui.message(MessageType::Warning, "You have mixed changes")?;
        }
        // if check(String::from("Do you want to stage all changes? (y/n)"))? {
        //     stage_all()?;
        //     cli.step_success("Staged all files", None);
        // } else {
        //     cli.step_success("Using staged changes", None);
        // }
        return Ok(());
    }

    // Literally nothing to commit.
    if !staged_changes && !unstaged_changes && !untracked_files {
        progress.set_message("No changes to staged".to_string());
        return Ok(());
    }

    Ok(())
}

/// Determines the commit message to use for the commit.
async fn get_commit_message(opts: &SaveOpts, tui: &Tui) -> Result<String> {
    if opts.ai {
        // Create a progress indicator for AI message generation
        let progress = if !opts.json_mode {
            tui.progress("Generating AI commit message")
        } else {
            tui.progress("")
        };

        // Generate a commit message using AI
        let message_result = commit_message().await;

        // Handle the result
        match message_result {
            Ok(message) => {
                progress.finish(&format!("AI commit message: {}", message));
                return Ok(message);
            }
            Err(e) => {
                progress.fail(&format!("AI commit message generation failed: {}", e));
                return Err(anyhow!("Failed to generate AI commit message: {}", e));
            }
        }
    }

    if opts.message.is_empty() {
        if opts.amend {
            // For amend, we can use an empty string as the previous message will be preserved
            if !opts.json_mode {
                tui.message(MessageType::Info, "Using previous commit message")?;
            }
            return Ok(String::new());
        } else {
            // This should not be reached due to the check above, but providing a fallback
            return Err(anyhow!(
                "Commit message is required. Use -m to provide a message, --ai to generate one, --empty for an empty commit, or --amend to amend the previous commit."
            ));
        }
    }
    Ok(opts.message.clone())
}
