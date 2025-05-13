use anyhow::{bail, Result};
use std::env;
use std::process::Command;
use toml::Value;

pub fn config_set(key: &str, value: &str) -> Result<()> {
    let manager = sage_config::ConfigManager::new()?;
    let mut cfg = manager.load()?;

    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["editor"] => cfg.editor = value.to_string(),
        ["auto_update"] => {
            cfg.auto_update = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid bool value for auto_update"))?
        }
        ["plugin_dirs"] => {
            cfg.plugin_dirs = value.split(',').map(|s| s.trim().to_string()).collect()
        }
        ["tui", "font_size"] => {
            cfg.tui.font_size = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid u32 value for tui.font_size"))?
        }
        ["tui", "color_theme"] => cfg.tui.color_theme = value.to_string(),
        ["tui", "line_numbers"] => {
            cfg.tui.line_numbers = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid bool value for tui.line_numbers"))?
        }
        ["ai", "model"] => cfg.ai.model = value.to_string(),
        ["ai", "api_url"] => cfg.ai.api_url = value.to_string(),
        ["ai", "api_key"] => cfg.ai.api_key = value.to_string(),
        ["ai", "max_tokens"] => {
            cfg.ai.max_tokens = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid u32 value for ai.max_tokens"))?
        }
        ["pull_requests", "enabled"] => {
            cfg.pull_requests.enabled = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid bool value for pull_requests.enabled"))?
        }
        ["pull_requests", "default_base"] => cfg.pull_requests.default_base = value.to_string(),
        ["pull_requests", "provider"] => cfg.pull_requests.provider = value.to_string(),
        ["pull_requests", "access_token"] => cfg.pull_requests.access_token = value.to_string(),
        ["extras", extra_key] => {
            cfg.extras
                .insert((*extra_key).to_string(), Value::String(value.to_string()));
        }
        ["ai", "extras", extra_key] => {
            cfg.ai
                .extras
                .insert((*extra_key).to_string(), Value::String(value.to_string()));
        }
        ["tui", "extras", extra_key] => {
            cfg.tui
                .extras
                .insert((*extra_key).to_string(), Value::String(value.to_string()));
        }
        ["pull_requests", "extras", extra_key] => {
            cfg.pull_requests
                .extras
                .insert((*extra_key).to_string(), Value::String(value.to_string()));
        }
        _ => bail!("Unknown or unsupported config key: {}", key),
    }
    manager.update(&cfg, true)?;
    println!("Set {} = {}", key, value);
    Ok(())
}
