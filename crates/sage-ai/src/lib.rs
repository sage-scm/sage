use anyhow::{Context, Result, anyhow};
use rig::{client::CompletionClient, completion::Prompt, providers::openai};
use std::env;
use std::time::Duration;

const MAX_RETRIES: usize = 3;

/// Asks the AI with a prompt
pub async fn ask(prompt: &str) -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY").context("OPENAI_API_KEY not set")?;
    let ai_model = env::var("OPENAI_MODEL").context("OPENAI_MODEL not set")?;
    let api_url = env::var("OPENAI_URL").context("OPENAI_URL not set")?;
    let timeout = env::var("OPENAI_TIMEOUT")
        .context("OPENAI_TIMEOUT not set")?
        .parse::<u64>()?;
    let max_tokens = env::var("OPENAI_MAX_TOKENS")
        .context("OPENAI_MAX_TOKENS not set")?
        .parse::<u64>()?;

    let trimmed_api_url = api_url.trim().trim_end_matches('/');

    let http_client_builder = if timeout > 0 {
        reqwest::Client::builder().timeout(Duration::from_secs(timeout))
    } else {
        reqwest::Client::builder()
    };

    let http_client = http_client_builder
        .build()
        .context("Failed to build HTTP client for AI Provider")?;

    let mut client_builder = openai::Client::builder(&api_key).custom_client(http_client);

    if !trimmed_api_url.is_empty() {
        client_builder = client_builder.base_url(trimmed_api_url);
    }

    let client = client_builder
        .build()
        .context("Failed to build OpenAI-compatible client")?;

    let mut agent_builder = client.agent(&ai_model);
    if max_tokens > 0 {
        agent_builder = agent_builder.max_tokens(max_tokens as u64);
    }
    let agent = agent_builder.build();

    // Retry logic
    let mut attempts = MAX_RETRIES;
    let mut last_error = None;
    while attempts > 0 {
        match tokio::time::timeout(Duration::from_secs(timeout), agent.prompt(prompt)).await {
            Ok(Ok(response)) => {
                if response.trim().is_empty() {
                    return Err(anyhow!("Empty response content"));
                }

                return Ok(response);
            }
            Ok(Err(e)) => {
                last_error = Some(anyhow::Error::from(e));
                println!("Error: {:?}", last_error);
            }
            Err(_) => {
                last_error = Some(anyhow!("Request timed out after {} seconds", timeout));
            }
        }

        attempts -= 1;
        if attempts == 0 {
            break;
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Err(last_error
        .unwrap_or_else(|| anyhow!("Failed to get response from AI provider"))
        .context("Failed to get caht completion"))
}
