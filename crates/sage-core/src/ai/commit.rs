use crate::ai::prompts;
use anyhow::Result;
use sage_git::repo;

pub async fn commit_message() -> Result<String> {
    // Limit diff size for faster processing - 12K allows for more context
    const MAX_DIFF_SIZE: usize = 12288;

    // Get enhanced diff with file context
    let mut diff = repo::diff()?;

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

    // Clean up response
    let res = res.trim();
    let res = if res.starts_with("```") && res.ends_with("```") {
        res.trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string()
    } else {
        res.to_string()
    };

    Ok(res)
}
