use crate::ai::prompts;
use anyhow::Result;
use sage_git::diff;

pub async fn commit_message() -> Result<String> {
    // Limit diff size for faster processing - 12K allows for more context
    const MAX_DIFF_SIZE: usize = 12288;

    // Get enhanced diff with file context
    let mut diff = diff::diff_ai()?;

    // Truncate diff if it's too large to speed up processing
    if diff.len() > MAX_DIFF_SIZE {
        // Keep the summary section intact and truncate only the diff content
        if let Some(diff_content_idx) = diff.find("# Diff Content") {
            let (summary, content) = diff.split_at(diff_content_idx);
            let remaining_size = MAX_DIFF_SIZE - summary.len();
            let truncated_content = content.chars().take(remaining_size).collect::<String>();
            diff = format!("{}{}\n[diff truncated]", summary, truncated_content);
        } else {
            // Fallback if the format is unexpected
            diff = diff.chars().take(MAX_DIFF_SIZE).collect::<String>() + "\n[diff truncated]";
        }
    }

    // Generate and send prompt
    let prompt = prompts::commit_message_prompt(&diff);
    let res = super::ask(&prompt).await?;

    // Clean up response - handle markdown code blocks with or without language specifiers
    let res = res.trim();
    let res = if res.starts_with("```") {
        // Find the first newline after the opening backticks to handle language specifiers
        let without_opening = res.trim_start_matches("```");
        let content = if let Some(newline_idx) = without_opening.find('\n') {
            // Skip language specifier line
            &without_opening[newline_idx + 1..]
        } else {
            without_opening
        };

        // Remove closing backticks
        if content.ends_with("```") {
            content.trim_end_matches("```").trim().to_string()
        } else {
            content.trim().to_string()
        }
    } else {
        res.to_string()
    };

    Ok(res)
}
