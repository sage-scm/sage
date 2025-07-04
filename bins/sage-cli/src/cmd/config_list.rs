use anyhow::Result;
use sage_config::ConfigManager;
use sage_core::CliOutput;
use std::collections::BTreeMap;

pub fn config_list() -> Result<()> {
    let cli = CliOutput::new();
    cli.header("config list");

    let manager = ConfigManager::new()?;
    let config = manager.load()?;

    println!("Global config: {}", manager.global_config_path().display());
    println!("Local config:  {}", manager.local_config_path().display());
    println!();

    let mut config_map = BTreeMap::new();

    // Root level config
    config_map.insert(
        "editor",
        format!("{}  # Default text editor", config.editor),
    );
    config_map.insert(
        "auto_update",
        format!("{}  # Check for updates automatically", config.auto_update),
    );
    config_map.insert(
        "plugin_dirs",
        format!(
            "{:?}  # Directories to look for plugins",
            config.plugin_dirs
        ),
    );

    // TUI config
    config_map.insert(
        "tui.font_size",
        format!("{}  # Font size for the TUI", config.tui.font_size),
    );
    config_map.insert(
        "tui.color_theme",
        format!("{}  # Color theme for the TUI", config.tui.color_theme),
    );
    config_map.insert(
        "tui.line_numbers",
        format!(
            "{}  # Show line numbers in the TUI",
            config.tui.line_numbers
        ),
    );

    // AI config
    config_map.insert(
        "ai.model",
        format!("{}  # Default AI model to use", config.ai.model),
    );
    config_map.insert(
        "ai.api_url",
        format!("{}  # API endpoint for the AI service", config.ai.api_url),
    );
    config_map.insert(
        "ai.max_tokens",
        format!(
            "{}  # Maximum tokens for AI responses",
            config.ai.max_tokens
        ),
    );
    config_map.insert(
        "ai.api_key",
        format!(
            "{}  # API key for LLM provider authentication",
            if config.ai.api_key.is_empty() {
                "<not set>".to_string()
            } else {
                "***".repeat(config.ai.api_key.len().min(20))
            }
        ),
    );

    // Pull Requests config
    config_map.insert(
        "pull_requests.enabled",
        format!(
            "{}  # Enable pull request integration",
            config.pull_requests.enabled
        ),
    );
    config_map.insert(
        "pull_requests.default_base",
        format!(
            "{}  # Default base branch for PRs",
            config.pull_requests.default_base
        ),
    );
    config_map.insert(
        "pull_requests.provider",
        format!(
            "{}  # Git provider (github/gitlab/etc)",
            config.pull_requests.provider
        ),
    );
    config_map.insert(
        "pull_requests.access_token",
        format!(
            "{}  # Personal access token for PR authentication",
            if config.pull_requests.access_token.is_empty() {
                "<not set>".to_string()
            } else {
                "***".repeat(config.pull_requests.access_token.len().min(20))
            }
        ),
    );

    // Save config
    config_map.insert(
        "save.ask_on_mixed_staging",
        format!(
            "{}  # Ask when both staged and unstaged changes exist",
            config.save.ask_on_mixed_staging
        ),
    );

    // Print all config values
    for (key, value) in config_map {
        println!("{:.<40} {}", key, value);
    }

    // Print extras if any
    if !config.extras.is_empty() {
        println!("\nAdditional custom settings:");
        for (key, value) in &config.extras {
            println!("  {}\\n    {}", key, value);
        }
    }

    // Print extras from nested configs
    let print_extras = |prefix: &str, extras: &std::collections::HashMap<String, toml::Value>| {
        if !extras.is_empty() {
            println!("\nAdditional {} settings:", prefix);
            for (key, value) in extras {
                println!("  {}.{}\\n    {}", prefix, key, value);
            }
        }
    };

    print_extras("tui.extras", &config.tui.extras);
    print_extras("ai.extras", &config.ai.extras);
    print_extras("pull_requests.extras", &config.pull_requests.extras);

    cli.summary();
    Ok(())
}
