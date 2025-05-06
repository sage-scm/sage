use anyhow::{anyhow, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use sage_git::{
    amend::{self, AmendOpts},
    branch::{get_current, is_clean, push, stage_all, unstage_all},
    commit::{self, commit_empty},
    status::{has_changes, has_staged_changes, has_unstaged_changes, has_untracked_files},
};
use std::{io::Read, time::Instant};

use crate::ai::commit::commit_message;

#[derive(Debug, Default)]
pub struct SaveOpts {
    /// The message to commit with
    pub message: Option<String>,
    /// Commit all changes
    pub all: bool,
    /// Commit only these paths
    pub paths: Option<Vec<String>>,
    /// Use AI to generate a commit message
    pub ai: bool,
    /// Amend the previous commit
    pub amend: bool,
    /// Push to remote
    pub push: bool,
    /// Create an empty git commit
    pub empty: bool,
}

pub async fn save(opts: &SaveOpts) -> Result<()> {
    println!("🌿  sage — save");
    let start = Instant::now();

    // Check if a message is required (do this early)
    if !opts.empty && !opts.amend && opts.message.is_none() && !opts.ai {
        return Err(anyhow!("Commit message is required. Use -m to provide a message, --ai to generate one, --empty for an empty commit, or --amend to amend the previous commit."));
    }

    // Early exit if working tree is clean and we're not creating an empty commit
    if is_clean()? && !opts.empty && !opts.amend {
        println!("⚠️  Working tree is clean");
        println!("Done in {:?}", start.elapsed());
        return Ok(());
    }

    // Handle staging
    stage_correct_files(opts)?;
    let commit_message = get_commit_message(opts).await?;

    if opts.empty && !opts.amend {
        // Create the empty commit and get the commit ID
        let commit_id = match commit_empty() {
            Ok(id) => id,
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
                println!("❌  Failed to create empty commit: {}", clean_error.red());
                println!("Done in {:?}", start.elapsed());
                return Ok(());
            }
        };

        push_changes(opts)?;
        println!("●   Write empty commit ✔ {}", commit_id.yellow());
        println!("Done in {:?}", start.elapsed());
        return Ok(());
    }

    if opts.amend {
        let amend_opts = AmendOpts {
            message: commit_message.clone(),
            empty: opts.empty,
            no_edit: (opts.empty && opts.message.is_none())
                || (opts.message.is_some() && !has_staged_changes()?), // Use no_edit if we're keeping the previous message
        };
        amend::amend(&amend_opts)?;
        println!("●   Amended previous commit ✔");
        push_changes(opts)?;
        println!("Done in {:?}", start.elapsed());
        return Ok(());
    }

    // Create the commit and get the commit ID
    let commit_id = match commit::commit(&commit_message) {
        Ok(id) => id,
        Err(e) => {
            // Extract the actual error message without the "Failed to commit: " prefix
            let error_msg = e.to_string();
            let clean_error = if error_msg.starts_with("Failed to commit: ") {
                error_msg
                    .trim_start_matches("Failed to commit: ")
                    .to_string()
            } else {
                error_msg
            };

            // Display a more helpful error message
            if clean_error.is_empty() {
                println!("❌  Failed to commit: No changes to commit (working directory clean)");
            } else {
                println!("❌  Failed to commit: {}", clean_error.red());
            }
            return Ok(());
        }
    };
    println!("●   Write commit ✔ {}", commit_id.yellow());

    // Handle push if requested
    push_changes(opts)?;

    println!("Done in {:?}", start.elapsed());
    Ok(())
}

/// Push changes to remote.
fn push_changes(opts: &SaveOpts) -> Result<()> {
    if !opts.push {
        return Ok(());
    }

    let branch = get_current()?;
    push(&branch, false)?;
    println!("●   Push to origin/{} ✔", branch);
    Ok(())
}

/// Determines the files to stage for the commit.
fn stage_correct_files(opts: &SaveOpts) -> Result<()> {
    let staged_changes = has_staged_changes()?;
    let unstaged_changes = has_unstaged_changes()?;
    let untracked_files = has_untracked_files()?;
    let changed_files = has_changes()?;

    // User wants everything staged no matter what.
    if opts.all {
        stage_all()?;
        println!("●   Staged all changes ✔");
        return Ok(());
    }

    // If the user provided files to commit.
    if let Some(paths) = &opts.paths {
        if !paths.is_empty() {
            // TODO: Implement staging specific files properly
            // For now, just stage all changes
            stage_all()?;
            println!("●   Staged all changes ✔");
            return Ok(());
        }
    }

    // User is amending the last commit without changes.
    if opts.amend && opts.empty {
        unstage_all()?;
        println!("●   Unstaged all changes ✔");
        return Ok(());
    }

    // No files changed, no need to stage.
    if opts.amend && opts.message.is_some() && !changed_files {
        return Ok(());
    }

    // User is amending last commit with changes.
    if opts.amend && !staged_changes {
        stage_all()?;
        println!("●   Staged all changes ✔");
        return Ok(());
    }

    // User only has unstaged / untracked changes.
    if !staged_changes && (unstaged_changes || untracked_files) {
        stage_all()?;
        println!("●   Staged all changes ✔");
        return Ok(());
    }

    // User has already staged their changes.
    if staged_changes && !unstaged_changes && !untracked_files {
        println!("●   Using staged changes ✔");
        return Ok(());
    }

    // User has both staged and unstaged/untracked changes.
    if staged_changes && unstaged_changes {
        println!("⚠️  You have mixed changes");
        println!("Do you want to stage all changes? (y/n)");
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        let answer = answer.trim();
        if answer.eq_ignore_ascii_case("y") {
            stage_all()?;
            println!("●   Staged all changes ✔");
        } else {
            println!("●   Only using staged changes ✔");
        }
        return Ok(());
    }

    // Literally nothing to commit.
    if !staged_changes && !unstaged_changes && !untracked_files {
        println!("⚠️  No changes staged");
        return Ok(());
    }

    Ok(())
}

/// Determines the commit message to use for the commit.
async fn get_commit_message(opts: &SaveOpts) -> Result<String> {
    if opts.message.is_none() {
        if opts.ai {
            // Create a spinner for AI message generation
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                    .template("{spinner:.blue} Generating AI commit message...")
                    .unwrap(),
            );
            spinner.enable_steady_tick(std::time::Duration::from_millis(80));

            // Generate a commit message using AI
            let message_result = commit_message().await;

            // Stop the spinner
            spinner.finish_and_clear();

            // Handle the result
            match message_result {
                Ok(message) => {
                    println!("●   AI commit message ✔ {}", message.clone().blue());
                    return Ok(message);
                }
                Err(e) => {
                    println!(
                        "❌   AI commit message generation failed: {}",
                        e.to_string().red()
                    );
                    return Err(anyhow!("Failed to generate AI commit message: {}", e));
                }
            }
        } else if opts.amend {
            // For amend, we can use an empty string as the previous message will be preserved
            println!("●   Using previous commit message ✔");
            return Ok(String::new());
        } else {
            // This should not be reached due to the check above, but providing a fallback
            return Err(anyhow!("Commit message is required. Use -m to provide a message, --ai to generate one, --empty for an empty commit, or --amend to amend the previous commit."));
        }
    }
    Ok(opts.message.clone().unwrap())
}
