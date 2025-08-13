use anyhow::Result;
use sage_git::{
    branch::stage_all,
    prelude::{git_ok, git_output, run_git},
};
use serde::Serialize;

#[derive(Debug, Default, Clone)]
pub struct SaveArgs {
    pub message: Option<String>,
    pub ai: bool,
    pub include: Vec<String>,
    pub json: bool,
    pub no_color: bool,
    pub amend: bool,
    pub push: bool,
    pub empty: bool,
}

#[derive(Debug, Serialize)]
struct SaveJsonOutput {
    action: &'static str,
    branch: String,
    staged: Vec<String>,
    commit: Option<CommitInfo>,
    status: &'static str,
    warning: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommitInfo {
    sha: String,
    title: String,
}

pub async fn save(ui: &crate::Ui, args: &SaveArgs) -> Result<i32> {
    let branch = sage_git::branch::get_current()?;

    let candidates = if !args.include.is_empty() {
        // Use only the provided changes
        args.include.clone()
    } else {
        // Use all the changes
        sage_git::branch::discover_changes()?
    };

    let staged_preview = if !args.include.is_empty() {
        sage_git::branch::list_files_matching(&args.include)?
    } else {
        let mut v = candidates.clone();
        v.sort();
        v
    };

    // Nothing to stage?
    if staged_preview.is_empty() {
        if args.json {
            let out = SaveJsonOutput {
                action: "commit",
                branch,
                staged: vec![],
                commit: None,
                status: "noop",
                warning: Some("nothing to commit".to_string()),
            };
            println!("{}", serde_json::to_string_pretty(&out)?);
            return Ok(0);
        }
        println!("Nothing to commit.");
    }

    // Draft message if needed
    let mut message = args.message.clone();
    let mut warn: Option<String> = None;
    if message.is_none() && args.ai {
        let draft = crate::ai::commit::commit_message()
            .await
            .unwrap_or_else(|_| {
                heuristic_title_from_diff().unwrap_or_else(|_| "chore: update files".into())
            });
        if args.json {
            message = Some(draft);
        } else {
            println!("AI Draft message:\n {}\n", draft);
            let choice =
                sage_tui::basic::prompt_line("Edit message? [e=edit / y=accept / n=reject]")?;
            match choice.as_str() {
                "e" | "E" => {
                    // message = Some(sage_tui::base::prompt_multiline_with_default(&draft)?);
                    message = Some(sage_tui::basic::prompt_editor(&draft)?);
                }
                "y" | "Y" => {
                    message = Some(draft);
                }
                _ => {
                    warn = Some("AI message rejected; procceding without -m".into());
                }
            }
        };
    }

    // If still none, request confirmation that empty is ok (git allows empty title but we discourage it)
    if message.is_none() && !args.json {
        println!("Commit preview");
        for f in &staged_preview {
            println!("  {}", f);
        }
        println!();
        println!("Message: <empty>");
        if !sage_tui::basic::confirm_yes_no("Proceed to commit with an empty message?")? {
            return Ok(2);
        }
    } else if !args.json {
        // Pretty preview with message
        println!("Commit preview");
        println!(" Staged files ({}):", staged_preview.len());
        for f in &staged_preview {
            println!("  {}", f);
        }
        println!(
            "\n Message:\n    {}\n",
            message.as_deref().unwrap_or("<empty>")
        );
        if !sage_tui::basic::confirm_yes_no("Proceed to commit?")? {
            return Ok(2);
        }
    }

    // Stage files
    if !args.include.is_empty() {
        // Use explicit paths
        git_ok(
            ["add"]
                .into_iter()
                .chain(args.include.iter().map(|s| s.as_str())),
        )?;
    } else {
        stage_all()?;
    }

    // Commit
    let commit_res = if let Some(msg) = &message {
        run_git(["commit", "-m", msg])?
    } else {
        run_git(["commit", "--allow-empty-message", "-m", ""])?
    };

    if !commit_res.status.success() {
        let stderr = String::from_utf8_lossy(&commit_res.stderr);
        if args.json {
            let out = SaveJsonOutput {
                action: "commit",
                branch,
                staged: staged_preview,
                commit: None,
                status: "error",
                warning: Some(format!("git commit failed: {}", stderr.trim())),
            };
            println!("{}", serde_json::to_string_pretty(&out)?);
        } else {
            eprintln!(
                "Error\n  git commit failed:\n  {}",
                sage_tui::indent(&stderr, 2)
            );
        }
        return Ok(4);
    }

    // get new commit sha and title
    let sha = sage_git::commit::last_commit_id()?;
    let title = if let Some(m) = &message {
        first_line(m)
    } else {
        "".into()
    };

    if args.json {
        let out = SaveJsonOutput {
            action: "commit",
            branch,
            staged: staged_preview,
            commit: Some(CommitInfo {
                sha: sha.clone(),
                title: title.clone(),
            }),
            status: "ok",
            warning: warn,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(0);
    }

    println!("Result\n  Created commit {}", sha);
    if let Some(w) = warn {
        println!("Warning\n  {}", w);
    }
    println!();
    Ok(0)
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").to_string()
}

fn heuristic_title_from_diff() -> Result<String> {
    // Very naive: look at changed top-level paths and produce a title
    let out = git_output(["diff", "--name-only"])?;
    let mut parts: Vec<String> = Vec::new();
    for p in out.lines().take(3) {
        let top = p.split('/').next().unwrap_or(p);
        if !parts.contains(&top.to_string()) {
            parts.push(top.to_string());
        }
    }
    if parts.is_empty() {
        Ok("chore: update".into())
    } else {
        Ok(format!("chore({}): update", parts.join(",")))
    }
}
