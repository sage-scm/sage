use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The main configuration structure for Sage.
///
/// Fields should be extended as new options become available.
/// All fields should be `Option<T>` to facilitate merging of layered configs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Name of the preferred text editor
    pub editor: Option<String>,



    /// Whether Sage should check for updates automatically
    pub auto_update: Option<bool>,

    /// List of plugin directories
    pub plugin_dirs: Option<Vec<String>>,

    /// Built-in and extension configuration data for the terminal user interface
    pub tui: Option<TuiConfig>,

    /// Configuration for AI model usage (LLM API, provider, key, etc.)
    pub ai: Option<AiConfig>,

    /// Configuration for pull request system
    pub pull_requests: Option<PrConfig>,

    /// Custom key-value configuration
    pub extras: Option<HashMap<String, toml::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TuiConfig {
    /// Font size for TUI
    pub font_size: Option<u32>,
    /// Custom color theme for the TUI
    pub color_theme: Option<String>,
    /// Whether to display line numbers
    pub line_numbers: Option<bool>,
    /// Any additional, TUI-specific settings (extensible)
    pub extras: Option<HashMap<String, toml::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AiConfig {
    /// AI model identifier (e.g., "gpt-4", "claude-3")
    pub model: Option<String>,
    /// API endpoint for the LLM provider
    pub api_url: Option<String>,
    /// API key for authenticating with the LLM provider
    pub api_key: Option<String>,
    /// Maximum tokens or request budget
    pub max_tokens: Option<u32>,
    /// Any additional, AI/LLM-specific settings
    pub extras: Option<HashMap<String, toml::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PrConfig {
    /// Whether to enable pull request integration
    pub enabled: Option<bool>,
    /// Default base branch for PRs
    pub default_base: Option<String>,
    /// Name or URL of remote repository provider (e.g., "github", "gitlab", or provider URL)
    pub provider: Option<String>,
    /// Personal access token or app ID for PR system authentication
    pub access_token: Option<String>,
    /// Extra/extension settings
    pub extras: Option<HashMap<String, toml::Value>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            editor: None,
            theme: None,
            auto_update: None,
            plugin_dirs: None,
            tui: None,
            ai: None,
            pull_requests: None,
            extras: None,
        }
    }
}

impl Config {
    /// Create an empty config (all fields None)
    pub fn empty() -> Self {
        Self::default()
    }

    /// Fill missing values in self with values from `defaults`
    pub fn merge_with(mut self, defaults: &Config) -> Self {
        if self.editor.is_none() {
            self.editor = defaults.editor.clone();
        }

        if self.auto_update.is_none() {
            self.auto_update = defaults.auto_update;
        }
        if self.plugin_dirs.is_none() {
            self.plugin_dirs = defaults.plugin_dirs.clone();
        }
        if self.tui.is_none() {
            self.tui = defaults.tui.clone();
        } else if let (Some(ref mut mine), Some(ref theirs)) = (&mut self.tui, &defaults.tui) {
            *mine = mine.clone().merge_with(theirs);
        }
        if self.ai.is_none() {
            self.ai = defaults.ai.clone();
        } else if let (Some(ref mut mine), Some(ref theirs)) = (&mut self.ai, &defaults.ai) {
            *mine = mine.clone().merge_with(theirs);
        }
        if self.pull_requests.is_none() {
            self.pull_requests = defaults.pull_requests.clone();
        } else if let (Some(ref mut mine), Some(ref theirs)) = (&mut self.pull_requests, &defaults.pull_requests) {
            *mine = mine.clone().merge_with(theirs);
        }
        if self.extras.is_none() {
            self.extras = defaults.extras.clone();
        } else if let (Some(ref mut mine), Some(ref theirs)) = (&mut self.extras, &defaults.extras) {
            for (k, v) in theirs {
                mine.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
        self
    }

    /// Provide sensible built-in defaults
    pub fn default_values() -> Self {
        Self {
            editor: Some("vim".to_owned()),

            auto_update: Some(true),
            plugin_dirs: Some(vec!["~/.config/sage/plugins".to_owned()]),
            tui: Some(TuiConfig {
                font_size: Some(14),
                color_theme: Some("default".to_string()),
                line_numbers: Some(true),
                extras: Some(HashMap::new()),
            }),
            ai: Some(AiConfig {
                model: Some("gpt-4".to_owned()),
                api_url: None,
                api_key: None,
                max_tokens: Some(4096),
                extras: Some(HashMap::new()),
            }),
            pull_requests: Some(PrConfig {
                enabled: Some(false),
                default_base: Some("main".to_string()),
                provider: None,
                access_token: None,
                extras: Some(HashMap::new()),
            }),
            extras: Some(HashMap::new()),
        }
    }
}

// Merge helpers for nested config types
impl TuiConfig {
    pub fn merge_with(self, defaults: &TuiConfig) -> TuiConfig {
        TuiConfig {
            font_size: self.font_size.or(defaults.font_size),
            color_theme: self.color_theme.clone().or(defaults.color_theme.clone()),
            line_numbers: self.line_numbers.or(defaults.line_numbers),
            extras: merge_maps(self.extras, defaults.extras.clone())
        }
    }
}
impl AiConfig {
    pub fn merge_with(self, defaults: &AiConfig) -> AiConfig {
        AiConfig {
            model: self.model.clone().or(defaults.model.clone()),
            api_url: self.api_url.clone().or(defaults.api_url.clone()),
            api_key: self.api_key.clone().or(defaults.api_key.clone()),
            max_tokens: self.max_tokens.or(defaults.max_tokens),
            extras: merge_maps(self.extras, defaults.extras.clone())
        }
    }
}
impl PrConfig {
    pub fn merge_with(self, defaults: &PrConfig) -> PrConfig {
        PrConfig {
            enabled: self.enabled.or(defaults.enabled),
            default_base: self.default_base.clone().or(defaults.default_base.clone()),
            provider: self.provider.clone().or(defaults.provider.clone()),
            access_token: self.access_token.clone().or(defaults.access_token.clone()),
            extras: merge_maps(self.extras, defaults.extras.clone())
        }
    }
}

// Helper for merging Option<HashMap<...>>
fn merge_maps(
    a: Option<HashMap<String, toml::Value>>,
    b: Option<HashMap<String, toml::Value>>,
) -> Option<HashMap<String, toml::Value>> {
    match (a, b) {
        (Some(mut a_map), Some(b_map)) => {
            for (k, v) in b_map {
                a_map.entry(k).or_insert(v);
            }
            Some(a_map)
        }
        (Some(a_map), None) => Some(a_map),
        (None, Some(b_map)) => Some(b_map),
        (None, None) => None,
    }
}