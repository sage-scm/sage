use serde::{Deserialize, Serialize};

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
    pub api_key: Option<String>,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default)]
    pub additional_commit_prompt: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: default_ai_provider(),
            api_key: None,
            model: default_model(),
            additional_commit_prompt: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_auto_stage")]
    pub auto_stage: bool,

    #[serde(default = "default_commit_template")]
    pub commit_template: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            auto_stage: default_auto_stage(),
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
    "openai".to_string()
}

fn default_model() -> String {
    "gpt-4".to_string()
}

fn default_auto_stage() -> bool {
    true
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
