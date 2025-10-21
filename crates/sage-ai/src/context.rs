use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use rig::providers::openai;
use sage_config::ConfigManager;
use std::time::Duration;

pub(crate) enum AiProvider {
    OpenAI {
        client: openai::Client,
    },
    Ollama {
        http: reqwest::Client,
        base_url: String,
    },
}

pub(crate) struct AiContext {
    pub(crate) provider: AiProvider,
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
        let manager = ConfigManager::load().context("Failed to load configuration")?;
        let config = manager.get();

        let provider_name = sanitize(config.ai.provider.clone()).to_lowercase();

        let ai_model = sanitize(config.ai.model.clone());

        if ai_model.is_empty() {
            anyhow::bail!("AI model not set. Please configure it in your sage config.");
        }

        let api_url = sanitize(config.ai.api_url.clone());
        let timeout_secs = config.ai.timeout_secs;
        let timeout_duration = Duration::from_secs(timeout_secs);
        let max_tokens = config.ai.max_tokens;
        let max_retries = config.ai.max_retries;
        let retry_delay_ms = config.ai.retry_delay_ms;

        let http_client_builder = if timeout_secs > 0 {
            reqwest::Client::builder().timeout(timeout_duration)
        } else {
            reqwest::Client::builder()
        };

        let http_client = http_client_builder
            .build()
            .context("Failed to build HTTP client for AI Provider")?;

        // Construct provider-specific clients
        let provider = match provider_name.as_str() {
            // OpenAI requires an API key
            "openai" => {
                let api_key = config
                    .ai
                    .api_key
                    .as_ref()
                    .map(|s| sanitize(s.expose().to_string()))
                    .filter(|value| !value.is_empty())
                    .context("AI API key not set. Please configure it in your sage config.")?;

                let mut client_builder =
                    openai::Client::<reqwest::Client>::builder(&api_key).with_client(http_client);

                let trimmed_api_url = api_url.trim_end_matches('/');
                if !trimmed_api_url.is_empty() {
                    client_builder = client_builder.base_url(trimmed_api_url);
                }

                let client = client_builder.build();
                AiProvider::OpenAI { client }
            }
            // Ollama talks to a local HTTP API; no API key
            "ollama" => {
                let base_url = api_url.trim_end_matches('/').to_string();
                if base_url.is_empty() {
                    // Common default for local Ollama
                    AiProvider::Ollama {
                        http: http_client,
                        base_url: "http://localhost:11434".to_string(),
                    }
                } else {
                    AiProvider::Ollama {
                        http: http_client,
                        base_url,
                    }
                }
            }
            other => anyhow::bail!("Unsupported AI provider: {}", other),
        };

        Ok(AiContext {
            provider,
            model: ai_model,
            timeout: timeout_duration,
            max_tokens: (max_tokens > 0).then_some(max_tokens),
            max_retries,
            retry_delay: Duration::from_millis(retry_delay_ms),
        })
    })
}
