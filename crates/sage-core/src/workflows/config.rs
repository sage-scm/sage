use anyhow::{Result, anyhow};
use sage_config::ConfigManager;
use sage_fmt::MessageType;

pub fn config(
    key: Option<String>,
    value: Option<String>,
    console: &sage_fmt::Console,
) -> Result<()> {
    match (key, value) {
        (Some(key), Some(value)) => {
            if value.trim().is_empty() {
                // Remove/unset config
                let mut manager = ConfigManager::load()?;
                set_config_value(&mut manager, &key, None)?;
                manager.save()?;
                console.message(MessageType::Success, "Unset config")?;
            } else {
                // Set config
                let mut manager = ConfigManager::load()?;
                set_config_value(&mut manager, &key, Some(&value))?;
                manager.save()?;
                console.message(MessageType::Success, "Set config")?;
            }
        }
        (Some(key), None) => {
            // Get single config value
            let manager = ConfigManager::load()?;
            let value = get_config_value(manager.get(), &key)?;
            println!("{}: {}", key, value);
        }
        (None, Some(_value)) => {
            console.message(MessageType::Error, "Can't set value without a key")?;
        }
        (None, None) => {
            // List all config values
            let manager = ConfigManager::load()?;
            let config = manager.get();

            println!("ai.provider: {}", config.ai.provider);
            if let Some(api_key) = &config.ai.api_key {
                println!("ai.api_key: {}", api_key);
            }
            println!("ai.model: {}", config.ai.model);
            println!("git.auto_stage: {}", config.git.auto_stage);
            println!("git.commit_template: {}", config.git.commit_template);
            println!("general.update_check: {}", config.general.update_check);
            println!("general.telemetry: {}", config.general.telemetry);
        }
    }

    Ok(())
}

fn get_config_value(config: &sage_config::SageConfig, key: &str) -> Result<String> {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.len() != 2 {
        return Err(anyhow!(
            "Invalid config key '{}'. Expected format: section.field (e.g., 'ai.provider')",
            key
        ));
    }

    match (parts[0], parts[1]) {
        ("ai", "provider") => Ok(config.ai.provider.clone()),
        ("ai", "api_key") => Ok(config.ai.api_key.clone().unwrap_or_default()),
        ("ai", "model") => Ok(config.ai.model.clone()),
        ("git", "auto_stage") => Ok(config.git.auto_stage.to_string()),
        ("git", "commit_template") => Ok(config.git.commit_template.clone()),
        ("general", "update_check") => Ok(config.general.update_check.to_string()),
        ("general", "telemetry") => Ok(config.general.telemetry.to_string()),
        _ => Err(anyhow!("Unknown config key: {}", key)),
    }
}

fn set_config_value(manager: &mut ConfigManager, key: &str, value: Option<&str>) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.len() != 2 {
        return Err(anyhow!(
            "Invalid config key '{}'. Expected format: section.field (e.g., 'ai.provider')",
            key
        ));
    }

    let config = manager.get_mut();

    match (parts[0], parts[1]) {
        ("ai", "provider") => {
            if let Some(v) = value {
                config.ai.provider = v.to_string();
            } else {
                return Err(anyhow!("Cannot unset required field 'ai.provider'"));
            }
        }
        ("ai", "api_key") => {
            config.ai.api_key = value.map(|v| v.to_string());
        }
        ("ai", "model") => {
            if let Some(v) = value {
                config.ai.model = v.to_string();
            } else {
                return Err(anyhow!("Cannot unset required field 'ai.model'"));
            }
        }
        ("git", "auto_stage") => {
            if let Some(v) = value {
                config.git.auto_stage = v
                    .parse::<bool>()
                    .map_err(|_| anyhow!("Invalid boolean value for 'git.auto_stage': {}", v))?;
            } else {
                return Err(anyhow!("Cannot unset required field 'git.auto_stage'"));
            }
        }
        ("git", "commit_template") => {
            if let Some(v) = value {
                config.git.commit_template = v.to_string();
            } else {
                return Err(anyhow!("Cannot unset required field 'git.commit_template'"));
            }
        }
        ("general", "update_check") => {
            if let Some(v) = value {
                config.general.update_check = v.parse::<bool>().map_err(|_| {
                    anyhow!("Invalid boolean value for 'general.update_check': {}", v)
                })?;
            } else {
                return Err(anyhow!(
                    "Cannot unset required field 'general.update_check'"
                ));
            }
        }
        ("general", "telemetry") => {
            if let Some(v) = value {
                config.general.telemetry = v
                    .parse::<bool>()
                    .map_err(|_| anyhow!("Invalid boolean value for 'general.telemetry': {}", v))?;
            } else {
                return Err(anyhow!("Cannot unset required field 'general.telemetry'"));
            }
        }
        _ => return Err(anyhow!("Unknown config key: {}", key)),
    }

    Ok(())
}
