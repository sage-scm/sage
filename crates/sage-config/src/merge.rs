use crate::model::*;

impl Config {
    pub fn merge_from(&mut self, other: &Self) {
        if !other.editor.is_empty() && other.editor != "code" {
            self.editor = other.editor.clone();
        }
        // Only override auto_update if it's explicitly set to false (non-default)
        if !other.auto_update {
            self.auto_update = other.auto_update;
        }
        if !other.plugin_dirs.is_empty() && other.plugin_dirs != vec!["plugins"] {
            self.plugin_dirs = other.plugin_dirs.clone();
        }
        self.save.merge_from(&other.save);
        self.tui.merge_from(&other.tui);
        self.ai.merge_from(&other.ai);
        self.pull_requests.merge_from(&other.pull_requests);
        self.extras.extend(other.extras.clone());
    }
}

impl SaveConfig {
    pub fn merge_from(&mut self, other: &Self) {
        // Only override ask_on_mixed_staging if it's explicitly set to false (non-default)
        if !other.ask_on_mixed_staging {
            self.ask_on_mixed_staging = other.ask_on_mixed_staging;
        }
    }
}

impl TuiConfig {
    pub fn merge_from(&mut self, other: &Self) {
        if other.font_size != 14 {
            self.font_size = other.font_size;
        }
        if !other.color_theme.is_empty() && other.color_theme != "SageDark" {
            self.color_theme = other.color_theme.clone();
        }
        // Only override line_numbers if it's explicitly set to false (non-default)
        if !other.line_numbers {
            self.line_numbers = other.line_numbers;
        }
        self.extras.extend(other.extras.clone());
    }
}

impl AiConfig {
    pub fn merge_from(&mut self, other: &Self) {
        if !other.model.is_empty() && other.model != "o4-mini" {
            self.model = other.model.clone();
        }
        if !other.api_url.is_empty() && other.api_url != "https://api.openai.com/v1/" {
            self.api_url = other.api_url.clone();
        }
        if !other.api_key.is_empty() {
            self.api_key = other.api_key.clone();
        }
        if other.timeout != 10 {
            self.timeout = other.timeout;
        }
        if other.max_tokens != 4096 {
            self.max_tokens = other.max_tokens;
        }
        self.extras.extend(other.extras.clone());
    }
}

impl PrConfig {
    pub fn merge_from(&mut self, other: &Self) {
        // Only override enabled if it's explicitly set to true (non-default)
        if other.enabled {
            self.enabled = other.enabled;
        }
        if !other.default_base.is_empty() && other.default_base != "main" {
            self.default_base = other.default_base.clone();
        }
        if !other.provider.is_empty() && other.provider != "github" {
            self.provider = other.provider.clone();
        }
        if !other.access_token.is_empty() {
            self.access_token = other.access_token.clone();
        }
        self.extras.extend(other.extras.clone());
    }
}
