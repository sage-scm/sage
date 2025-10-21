use serde::{Deserialize, Serialize};

use crate::SecretString;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SageConfig {
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_ai_provider")]
    pub provider: String,

    #[serde(default)]
    pub api_key: Option<SecretString>,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default = "default_api_url")]
    pub api_url: String,

    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: u64,

    #[serde(default = "default_max_retries")]
    pub max_retries: usize,

    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,

    #[serde(default)]
    pub additional_commit_prompt: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: default_ai_provider(),
            api_key: None,
            model: default_model(),
            api_url: default_api_url(),
            timeout_secs: default_timeout_secs(),
            max_tokens: default_max_tokens(),
            max_retries: default_max_retries(),
            retry_delay_ms: default_retry_delay_ms(),
            additional_commit_prompt: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_auto_stage")]
    pub auto_stage: bool,

    #[serde(default = "default_disable_intermittent_fetch")]
    pub disable_intermittent_fetch: bool,

    #[serde(default = "default_commit_template")]
    pub commit_template: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            auto_stage: default_auto_stage(),
            disable_intermittent_fetch: default_disable_intermittent_fetch(),
            commit_template: default_commit_template(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_update_check")]
    pub update_check: bool,

    #[serde(default = "default_telemetry")]
    pub telemetry: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            update_check: default_update_check(),
            telemetry: default_telemetry(),
        }
    }
}

fn default_ai_provider() -> String {
    // Default remains OpenAI; users can set to "ollama"
    "openai".to_string()
}

fn default_model() -> String {
    "gpt-4".to_string()
}

pub const SECRET_KEYS: &[&str] = &["ai.api_key"];

fn default_api_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_timeout_secs() -> u64 {
    60
}

fn default_max_tokens() -> u64 {
    2_048
}

fn default_max_retries() -> usize {
    1
}

fn default_retry_delay_ms() -> u64 {
    0
}

fn default_auto_stage() -> bool {
    true
}

fn default_disable_intermittent_fetch() -> bool {
    false
}

fn default_commit_template() -> String {
    "feat: {summary}".to_string()
}

fn default_update_check() -> bool {
    true
}

fn default_telemetry() -> bool {
    false
}
