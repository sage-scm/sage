use anyhow::{Result, anyhow};
use rig::{client::CompletionClient, completion::Prompt};

mod commit;
mod context;
mod prompts;

use context::ai_context;

pub use commit::commit_message;

/// Asks the AI with a prompt
pub async fn ask(prompt: &str) -> Result<String> {
    let context = ai_context()?;

    let mut agent_builder = context.client.agent(&context.model);
    if let Some(max_tokens) = context.max_tokens {
        agent_builder = agent_builder.max_tokens(max_tokens);
    }
    let agent = agent_builder.build();

    // Retry logic
    let mut attempts = context.max_retries;
    let mut last_error = None;
    while attempts > 0 {
        match tokio::time::timeout(context.timeout, agent.prompt(prompt)).await {
            Ok(Ok(response)) => {
                // Extract the actual content from the response
                let content = response.to_string();
                if content.trim().is_empty() {
                    last_error = Some(anyhow!("AI provider returned empty response"));
                } else {
                    return Ok(content);
                }
            }
            Ok(Err(e)) => {
                last_error = Some(anyhow!("AI request failed: {}", e));
            }
            Err(_) => {
                last_error = Some(anyhow!(
                    "Request timed out after {} seconds",
                    context.timeout.as_secs()
                ));
            }
        }

        attempts -= 1;
        if attempts == 0 {
            break;
        }

        if !context.retry_delay.is_zero() {
            tokio::time::sleep(context.retry_delay).await;
        }
    }

    Err(last_error
        .unwrap_or_else(|| anyhow!("Failed to get response from AI provider"))
        .context("Failed to get chat completion"))
}
