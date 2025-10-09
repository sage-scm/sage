use crate::prompts;
use anyhow::{Result, anyhow};

const MAX_DIFF_SIZE: usize = 12_288;

pub async fn commit_message(diff: &str) -> Result<String> {
    let diff = diff.trim();
    if diff.is_empty() {
        return Ok("chore: no functional changes".to_string());
    }

    let diff_for_prompt = if diff.len() > MAX_DIFF_SIZE {
        let summary = summarize_diff(diff);
        let remaining_size = MAX_DIFF_SIZE.saturating_sub(summary.len().min(MAX_DIFF_SIZE / 2));
        let content: String = diff.chars().take(remaining_size).collect();
        format!("{summary}\n{content}\n[diff truncated]")
    } else {
        diff.to_string()
    };

    let prompt = prompts::commit_message_prompt(&diff_for_prompt);
    let res = super::ask(&prompt).await?;
    let res = clean_response(res)?;

    if res.trim().is_empty() {
        return Err(anyhow!("AI returned an empty response"));
    }

    if !is_valid_commit_message(&res) {
        return Err(anyhow!(
            "AI response was not a valid commit message: {}",
            res
        ));
    }

    Ok(res)
}

fn summarize_diff(diff: &str) -> String {
    let file_count = diff.lines().filter(|l| l.starts_with("+++")).count();
    let line_count = diff
        .lines()
        .filter(|l| l.starts_with('+') || l.starts_with('-'))
        .count();
    format!("# Summary: modified {file_count} files, {line_count} lines changed")
}

fn clean_response(res: String) -> Result<String> {
    let res = res.trim();
    let cleaned = if res.starts_with("```") {
        let without_opening = res.trim_start_matches("```");
        let content = without_opening
            .find('\n')
            .map_or(without_opening, |idx| &without_opening[idx + 1..]);
        content.trim_end_matches("```").trim().to_string()
    } else {
        res.to_string()
    };

    Ok(cleaned)
}

fn is_valid_commit_message(msg: &str) -> bool {
    let types = [
        "feat", "fix", "docs", "style", "chore", "refactor", "test", "ci",
    ];

    let lines: Vec<&str> = msg.lines().collect();
    if lines.is_empty() {
        return false;
    }

    let first_line = lines[0];
    let parts: Vec<&str> = first_line.splitn(2, ':').collect();
    if parts.len() != 2 || parts[1].trim().is_empty() {
        return false;
    }

    let type_scope = parts[0];

    let type_name = if let Some(paren_pos) = type_scope.find('(') {
        if let Some(close_paren) = type_scope.find(')') {
            if close_paren <= paren_pos {
                return false;
            }
            let after_close = &type_scope[close_paren + 1..];
            if !after_close.is_empty() && after_close != "!" {
                return false;
            }
            &type_scope[..paren_pos]
        } else {
            return false;
        }
    } else {
        type_scope.trim_end_matches('!')
    };

    if !types.contains(&type_name) {
        return false;
    }

    if first_line.len() > 72 {
        return false;
    }

    if lines.len() > 1 && !lines[1].is_empty() {
        return false;
    }

    true
}
