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
    format!("# Summar: Modified {file_count} files, {line_count} lines changed")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_single_line_commits() {
        assert!(is_valid_commit_message("feat: add new feature"));
        assert!(is_valid_commit_message("fix: resolve issue"));
        assert!(is_valid_commit_message("feat(api): add endpoint"));
        assert!(is_valid_commit_message("feat(auth)!: replace token system"));
        assert!(is_valid_commit_message("fix!: critical security patch"));
    }

    #[test]
    fn test_valid_multi_line_commits() {
        assert!(is_valid_commit_message(
            "refactor(sage-git)!: migrate to Git wrapper API\n\
            \n\
            BREAKING CHANGE: sage-git public API now uses GitResult instead of Result"
        ));

        assert!(is_valid_commit_message(
            "feat: add new authentication system\n\
            \n\
            This adds JWT support to the application"
        ));
    }

    #[test]
    fn test_invalid_commits() {
        assert!(!is_valid_commit_message(""));
        assert!(!is_valid_commit_message("no colon here"));
        assert!(!is_valid_commit_message("invalid: "));
        assert!(!is_valid_commit_message("feat:"));
        assert!(!is_valid_commit_message("wrongtype: message"));
        assert!(!is_valid_commit_message(
            "feat: second line not empty\n\
            this should have an empty line"
        ));
    }

    #[test]
    fn test_long_first_line() {
        let long_msg = format!("feat: {}", "x".repeat(100));
        assert!(!is_valid_commit_message(&long_msg));
    }
}
