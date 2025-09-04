use anyhow::Result;
use std::{fs, path::PathBuf};

use crate::ai::{ask, prompts};
use sage_git::{
    branch::{get_current, get_default_branch},
    helpers::get_commit_messages_between,
    status::status,
};
use sage_tui::Tui;

pub async fn share_branch(tui: &Tui) -> Result<()> {
    let repo_root = sage_git::repo::get_repo_root()?;
    let graph = sage_graph::SageGraph::load_or_default()?;
    let current_branch = get_current()?;
    let parent_branch = if graph.tracks(&current_branch) {
        match graph.info(&current_branch) {
            Some(info) => info.parent.clone(),
            None => get_default_branch()?,
        }
    } else {
        get_default_branch()?
    };

    let status = status()?;
    if !status.is_clean() {
        tui.message(
            sage_tui::MessageType::Warning,
            "Unstaged changes are not being pushed",
        )?;
    }

    let template_progress = tui.progress("Searching for template");
    let template = get_pull_request_template(&repo_root)?;
    template_progress.done();

    if template.is_empty() {
        tui.message(sage_tui::MessageType::Info, "No template found")?;
    } else {
        tui.message(sage_tui::MessageType::Success, "Template found")?;
    }

    let commits_progress = tui.progress("Gathering commits");
    let commits = get_commit_messages_between(&parent_branch, &current_branch)?;
    commits_progress.done();

    if commits.is_empty() {
        tui.message(
            sage_tui::MessageType::Warning,
            &format!(
                "No commits found between {} and {}",
                parent_branch, current_branch
            ),
        )?;
        return Ok(());
    }

    tui.message(
        sage_tui::MessageType::Info,
        &format!("Found {} commits to include in PR", commits.len()),
    )?;

    let ai_progress = tui.progress("Generating PR content with AI");
    let (title, body) = generate_ai_content(commits, template).await?;
    ai_progress.done();

    tui.header("Generated PR Content")?;
    tui.message(sage_tui::MessageType::Success, "PR title generated")?;
    println!("Title: {}", title);
    println!();
    tui.message(sage_tui::MessageType::Success, "PR body generated")?;
    println!("Body:\n{}", body);

    Ok(())
}

/// Gets the template to use for the pull request
fn get_pull_request_template(repo_root: &str) -> Result<String> {
    let github_pull_request_template_file = PathBuf::from(repo_root)
        .join(".github")
        .join("pull_request_template.md");

    let sage_pull_request_template_file = PathBuf::from(repo_root)
        .join(".sage")
        .join("pull_request_template.md");

    if github_pull_request_template_file.exists() {
        let contents = fs::read_to_string(github_pull_request_template_file)?;
        return Ok(contents);
    }

    if sage_pull_request_template_file.exists() {
        let contents = fs::read_to_string(sage_pull_request_template_file)?;
        return Ok(contents);
    }

    Ok(String::new())
}

/// Generates the pull request title and body using AI
async fn generate_ai_content(commits: Vec<String>, template: String) -> Result<(String, String)> {
    if commits.is_empty() {
        return Ok((
            "chore: no commits found".to_string(),
            "No commits were found for this branch.".to_string(),
        ));
    }

    // Generate PR title
    let title_prompt = prompts::pr_title_prompt(commits.clone()).await;
    let title = ask(&title_prompt).await?.trim().to_string();

    // Generate PR body
    let body_prompt = prompts::pr_description_prompt(&title, commits, &template);
    let body = ask(&body_prompt).await?.trim().to_string();

    Ok((title, body))
}
