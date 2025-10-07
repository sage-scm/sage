use anyhow::Result;
use sage_fmt::MessageType;

use crate::{get_config, list_config, remove_config, set_config};

pub fn config(
    key: Option<String>,
    value: Option<String>,
    console: &sage_fmt::Console,
) -> Result<()> {
    match (key, value) {
        (Some(key), Some(value)) => {
            if value.trim().is_empty() {
                remove_config(key)?;
                console.message(MessageType::Success, "Unset config")?;
            } else {
                set_config(key, value)?;
                console.message(MessageType::Success, "Set config")?;
            }
        }
        (Some(key), None) => {
            let value = get_config(Some(key.clone()))?;
            println!("{}: {}", key, value.unwrap_or_default());
        }
        (None, Some(_value)) => {
            console.message(MessageType::Error, "Can't set value without a key")?;
        }
        (None, None) => {
            let config = list_config()?;
            for key in config {
                let value = get_config(Some(key.clone()))?;
                println!("{}: {}", key, value.unwrap_or_default());
            }
        }
    }

    Ok(())
}
