use anyhow::{Result, anyhow};
use sage_config::{ConfigEntry, ConfigManager};
use sage_fmt::MessageType;

pub fn config(
    key: Option<String>,
    value: Option<String>,
    console: &sage_fmt::Console,
) -> Result<()> {
    match (key, value) {
        (Some(key), Some(value)) => {
            let mut manager = ConfigManager::load()?;
            if value.trim().is_empty() {
                sage_config::set_value(manager.get_mut(), &key, None)?;
                console.message(MessageType::Success, "Unset config")?;
            } else {
                sage_config::set_value(manager.get_mut(), &key, Some(&value))?;
                console.message(MessageType::Success, "Set config")?;
            }
            manager.save()?;
        }
        (Some(key), None) => {
            let manager = ConfigManager::load()?;
            match sage_config::get_entry(manager.get(), &key)? {
                Some(entry) => {
                    if let Some(raw) = entry.raw_value {
                        println!("{}: {}", key, raw);
                    } else {
                        println!("{}: <unset>", key);
                    }
                }
                None => {
                    return Err(anyhow!("Unknown config key: {}", key));
                }
            }
        }
        (None, Some(_value)) => {
            console.message(MessageType::Error, "Can't set value without a key")?;
        }
        (None, None) => {
            let manager = ConfigManager::load()?;
            let entries = sage_config::list_entries(manager.get())?;
            for ConfigEntry {
                key, display_value, ..
            } in entries
            {
                if let Some(display) = display_value {
                    println!("{key}: {display}");
                }
            }
        }
    }

    Ok(())
}
