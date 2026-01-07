use anyhow::{Context, Result, anyhow};
use once_cell::sync::OnceCell;
use rig::providers::openai;
use sage_config::ConfigManager;
use std::time::Duration;

pub(crate) struct AiContext {
    pub(crate) client: openai::Client,
    pub(crate) model: String,
    pub(crate) timeout: Duration,
    pub(crate) max_tokens: Option<u64>,
    pub(crate) max_retries: usize,
    pub(crate) retry_delay: Duration,
    pub(crate) reasoning_effort: Option<String>,
}

static AI_CONTEXT: OnceCell<AiContext> = OnceCell::new();

fn sanitize(value: String) -> String {
    value
        .trim()
        .trim_start_matches('=')
        .trim_matches('"')
        .trim()
        .to_string()
}

pub(crate) fn ai_context() -> Result<&'static AiContext> {
    AI_CONTEXT.get_or_try_init(|| {
        let manager = ConfigManager::load().context("Failed to load configuration")?;
        let config = manager.get();

        let ai_model = sanitize(config.ai.model.clone());

        if ai_model.is_empty() {
            anyhow::bail!("AI model not set. Please configure it in your sage config.");
        }

        let api_url = sanitize(config.ai.api_url.clone());

        let api_key = config
            .ai
            .api_key
            .as_ref()
            .map(|s| sanitize(s.expose().to_string()))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                anyhow!("AI API key not set. Please configure ai.api_key in your sage config.")
            })?;

        let timeout_secs = config.ai.timeout_secs;
        let timeout_duration = Duration::from_secs(timeout_secs);
        let max_tokens = config.ai.max_tokens;
        let max_retries = config.ai.max_retries;
        let retry_delay_ms = config.ai.retry_delay_ms;
        let reasoning_effort = config
            .ai
            .reasoning_effort
            .clone()
            .map(sanitize)
            .filter(|s| !s.eq_ignore_ascii_case("none"));

        println!(
            "Decided reasoning effort: {}",
            reasoning_effort.clone().unwrap_or_default()
        );

        let mut client_builder = openai::Client::builder().api_key(&api_key);

        let trimmed_api_url = api_url.trim_end_matches('/');
        if !trimmed_api_url.is_empty() {
            client_builder = client_builder.base_url(trimmed_api_url);
        }

        let client = client_builder
            .build()
            .context("Failed to build OpenAI client")?;

        Ok(AiContext {
            client,
            model: ai_model,
            timeout: timeout_duration,
            max_tokens: (max_tokens > 0).then_some(max_tokens),
            max_retries,
            retry_delay: Duration::from_millis(retry_delay_ms),
            reasoning_effort,
        })
    })
}
