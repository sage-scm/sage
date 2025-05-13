use crate::model::*;

impl Config {
    pub fn merge_from(&mut self, other: &Self) {
        if !other.editor.is_empty() {
            self.editor = other.editor.clone();
        }
        self.auto_update = other.auto_update;
        if !other.plugin_dirs.is_empty() {
            self.plugin_dirs = other.plugin_dirs.clone();
        }
        self.tui.merge_from(&other.tui);
        self.ai.merge_from(&other.ai);
        self.pull_requests.merge_from(&other.pull_requests);
        self.extras.extend(other.extras.clone());
    }
}

impl TuiConfig {
    pub fn merge_from(&mut self, other: &Self) {
        if other.font_size != 14 {
            self.font_size = other.font_size;
        }
        if !other.color_theme.is_empty() {
            self.color_theme = other.color_theme.clone();
        }
        self.line_numbers = other.line_numbers;
        self.extras.extend(other.extras.clone());
    }
}

impl AiConfig {
    pub fn merge_from(&mut self, other: &Self) {
        if !other.model.is_empty() {
            self.model = other.model.clone();
        }
        if !other.api_url.is_empty() {
            self.api_url = other.api_url.clone();
        }
        if !other.api_key.is_empty() {
            self.api_key = other.api_key.clone();
        }
        if other.max_tokens != 4096 {
            self.max_tokens = other.max_tokens;
        }
        self.extras.extend(other.extras.clone());
    }
}

impl PrConfig {
    pub fn merge_from(&mut self, other: &Self) {
        self.enabled = other.enabled;
        if !other.default_base.is_empty() {
            self.default_base = other.default_base.clone();
        }
        if !other.provider.is_empty() {
            self.provider = other.provider.clone();
        }
        if !other.access_token.is_empty() {
            self.access_token = other.access_token.clone();
        }
        self.extras.extend(other.extras.clone());
    }
}
