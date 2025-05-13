use std::collections::HashMap;

// Sensible defaults for each config.
use crate::model::*;

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: String::from("code"),
            auto_update: true,
            plugin_dirs: vec![String::from("plugins")],
            tui: TuiConfig::default(),
            ai: AiConfig::default(),
            pull_requests: PrConfig::default(),
            extras: HashMap::new(),
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            font_size: 14,
            color_theme: String::from("SageDark"),
            line_numbers: true,
            extras: HashMap::new(),
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            model: String::from("o4-mini"),
            api_url: String::from("https://api.openai.com/v1/"),
            api_key: String::from(""),
            max_tokens: 4096,
            extras: HashMap::new(),
        }
    }
}

impl Default for PrConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_base: String::from("main"),
            provider: String::from("github"),
            access_token: String::from(""),
            extras: HashMap::new(),
        }
    }
}
