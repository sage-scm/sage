use anyhow::{Context, Result, anyhow};
use once_cell::sync::OnceCell;
use rig::{client::CompletionClient, completion::Prompt, providers::openai};
use std::env;
use std::time::Duration;

mod commit;
mod prompts;

pub use commit::commit_message;

const DEFAULT_TIMEOUT_SECS: u64 = 60;
const DEFAULT_MAX_TOKENS: u64 = 2_048;
const DEFAULT_MAX_RETRIES: usize = 1;
const DEFAULT_RETRY_DELAY_MS: u64 = 0;

struct AiContext {
    client: openai::Client,
    model: String,
    timeout: Duration,
    max_tokens: Option<u64>,
    max_retries: usize,
    retry_delay: Duration,
}

static AI_CONTEXT: OnceCell<AiContext> = OnceCell::new();

fn sanitize(value: String) -> String {
    let trimmed = value.trim();
    trimmed
        .strip_prefix('=')
        .map(str::trim)
        .unwrap_or(trimmed)
        .trim_matches('"')
        .trim()
        .to_string()
}

fn ai_context() -> Result<&'static AiContext> {
    AI_CONTEXT.get_or_try_init(|| {
        let api_key = sanitize(env::var("OPENAI_API_KEY").context("OPENAI_API_KEY not set")?);
        let ai_model = sanitize(env::var("OPENAI_MODEL").context("OPENAI_MODEL not set")?);
        let api_url = sanitize(env::var("OPENAI_URL").context("OPENAI_URL not set")?);

        let timeout_secs = env::var("OPENAI_TIMEOUT")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS);
        let timeout_duration = Duration::from_secs(timeout_secs);

        let max_tokens = env::var("OPENAI_MAX_TOKENS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_MAX_TOKENS);

        let max_retries = env::var("OPENAI_MAX_RETRIES")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|&retries| retries > 0)
            .unwrap_or(DEFAULT_MAX_RETRIES);

        let retry_delay_ms = env::var("OPENAI_RETRY_DELAY_MS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_RETRY_DELAY_MS);

        let trimmed_api_url = api_url.trim();
        let http_client_builder = if timeout_secs > 0 {
            reqwest::Client::builder().timeout(Duration::from_secs(timeout_secs))
        } else {
            reqwest::Client::builder()
        };

        let http_client = http_client_builder
            .build()
            .context("Failed to build HTTP client for AI Provider")?;

        let mut client_builder = openai::Client::builder(&api_key).custom_client(http_client);

        let trimmed_api_url = trimmed_api_url.trim_end_matches('/');
        if !trimmed_api_url.is_empty() {
            client_builder = client_builder.base_url(trimmed_api_url);
        }

        let client = client_builder
            .build()
            .context("Failed to build OpenAI-compatible client")?;

        Ok(AiContext {
            client,
            model: ai_model,
            timeout: timeout_duration,
            max_tokens: (max_tokens > 0).then_some(max_tokens),
            max_retries,
            retry_delay: Duration::from_millis(retry_delay_ms),
        })
    })
}

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
                if response.trim().is_empty() {
                    last_error = Some(anyhow!("AI provider returned empty response"));
                } else {
                    return Ok(response);
                }
            }
            Ok(Err(e)) => {
                last_error = Some(anyhow::Error::from(e));
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
