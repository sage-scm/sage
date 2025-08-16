use crate::ai::prompts;
use anyhow::{Result, anyhow};
use sage_git::diff;

pub async fn commit_message() -> Result<String> {
    // Limit diff size for faster processing - 12K allows for more context
    const MAX_DIFF_SIZE: usize = 12288;

    // Get enhanced diff with file context
    let mut diff = diff::diff_ai()?;
    if diff.trim().is_empty() {
        return Ok("chore: no functional changes".to_string());
    }

    // Truncate diff if it's too large to speed up processing
    if diff.len() > MAX_DIFF_SIZE {
        let summary = summarize_diff(&diff);
        let remaining_size = MAX_DIFF_SIZE - summary.len().min(MAX_DIFF_SIZE / 2);
        let content = diff.chars().take(remaining_size).collect::<String>();
        diff = format!("{summary}\n{content}\n[diff truncated]");
    }

    // Generate and send prompt
    let prompt = prompts::commit_message_prompt(&diff);
    let res = super::ask(&prompt).await?;

    // Validate and clean response
    let res = clean_response(res)?;
    if !is_valid_commit_message(&res) {
        return Err(anyhow!("Invalid commit message format: {}", res));
    }

    Ok(res)
}

fn summarize_diff(diff: &str) -> String {
    let file_count = diff.lines().filter(|l| l.starts_with("+++")).count();
    let line_count = diff
        .lines()
        .filter(|l| l.starts_with('+') || l.starts_with('-'))
        .count();
    format!(
        "# Summar: Modified {file_count} files, {line_count} lines changed"
    )
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
        "feat", "fix", "docs", "style", "chore", "refactor", "test", "ci", "chore",
    ];
    let parts: Vec<&str> = msg.splitn(2, ':').collect();
    if parts.len() != 2 || parts[1].trim().is_empty() {
        return false;
    }

    let type_scope = parts[0];
    let _has_bang = type_scope.ends_with('!');
    let type_part = type_scope.trim_end_matches('!');
    let type_scope_parts: Vec<&str> = type_part.splitn(2, '(').collect();
    let type_name = type_scope_parts[0];
    types.contains(&type_name) && parts[1].len() <= 72
}
