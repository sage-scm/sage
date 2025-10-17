use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use rig::providers::openai;
use sage_config::ConfigManager;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 60;
const DEFAULT_MAX_TOKENS: u64 = 2_048;
const DEFAULT_MAX_RETRIES: usize = 1;
const DEFAULT_RETRY_DELAY_MS: u64 = 0;
const DEFAULT_API_URL: &str = "https://api.openai.com/v1";

pub(crate) struct AiContext {
    pub(crate) client: openai::Client,
    pub(crate) model: String,
    pub(crate) timeout: Duration,
    pub(crate) max_tokens: Option<u64>,
    pub(crate) max_retries: usize,
    pub(crate) retry_delay: Duration,
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

pub(crate) fn ai_context() -> Result<&'static AiContext> {
    AI_CONTEXT.get_or_try_init(|| {
        let manager = ConfigManager::load()?;
        let config = manager.get();
        
        let api_key = config
            .ai
            .api_key
            .as_ref()
            .map(|s| sanitize(s.clone()))
            .filter(|value| !value.is_empty())
            .expect("AI API key not set");
        
        let ai_model = sanitize(config.ai.model.clone());
        
        // Note: api_url, timeout, max_tokens, max_retries, and retry_delay_ms are not in the current
        // typed config. Using defaults for now. These should be added to the config struct if needed.
        let api_url = DEFAULT_API_URL.to_string();
        let timeout_secs = DEFAULT_TIMEOUT_SECS;
        let timeout_duration = Duration::from_secs(timeout_secs);
        let max_tokens = DEFAULT_MAX_TOKENS;
        let max_retries = DEFAULT_MAX_RETRIES;
        let retry_delay_ms = DEFAULT_RETRY_DELAY_MS;

        let http_client_builder = if timeout_secs > 0 {
            reqwest::Client::builder().timeout(Duration::from_secs(timeout_secs))
        } else {
            reqwest::Client::builder()
        };

        let http_client = http_client_builder
            .build()
            .context("Failed to build HTTP client for AI Provider")?;

        let mut client_builder =
            openai::Client::<reqwest::Client>::builder(&api_key).with_client(http_client);

        let trimmed_api_url = api_url.trim_end_matches('/');
        if !trimmed_api_url.is_empty() {
            client_builder = client_builder.base_url(trimmed_api_url);
        }

        let client = client_builder.build();

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
