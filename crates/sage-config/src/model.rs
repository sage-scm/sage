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
    pub editor: String,

    /// Whether Sage should check for updates automatically
    pub auto_update: bool,

    /// List of plugin directories
    pub plugin_dirs: Vec<String>,

    /// Save config and options
    pub save: SaveConfig,

    /// Built-in and extension configuration data for the terminal user interface
    pub tui: TuiConfig,

    /// Configuration for AI model usage (LLM API, provider, key, etc.)
    pub ai: AiConfig,

    /// Configuration for pull request system
    pub pull_requests: PrConfig,

    /// Custom key-value configuration
    pub extras: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveConfig {
    /// Weather to ask if you want to add all files to staging on a mixed stage detection
    pub ask_on_mixed_staging: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TuiConfig {
    /// Font size for TUI
    pub font_size: u32,
    /// Custom color theme for the TUI
    pub color_theme: String,
    /// Whether to display line numbers
    pub line_numbers: bool,
    /// Any additional, TUI-specific settings (extensible)
    pub extras: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    /// AI model identifier (e.g., "gpt-4", "claude-3")
    pub model: String,
    /// API endpoint for the LLM provider
    pub api_url: String,
    /// API key for authenticating with the LLM provider
    pub api_key: String,
    /// Maximum tokens or request budget
    pub max_tokens: u32,
    /// Any additional, AI/LLM-specific settings
    pub extras: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PrConfig {
    /// Whether to enable pull request integration
    pub enabled: bool,
    /// Default base branch for PRs
    pub default_base: String,
    /// Name or URL of remote repository provider (e.g., "github", "gitlab", or provider URL)
    pub provider: String,
    /// Personal access token or app ID for PR system authentication
    pub access_token: String,
    /// Extra/extension settings
    pub extras: HashMap<String, toml::Value>,
}

impl Config {
    /// Create an empty config (all fields None)
    pub fn empty() -> Self {
        Self::default()
    }
}
