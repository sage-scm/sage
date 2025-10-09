use anyhow::Result;
use sage_config::Config;

pub fn get_config(key: Option<String>) -> Result<Option<String>> {
    let config = Config::load()?;

    if let Some(key) = key {
        return config.get(&key);
    }

    Ok(None)
}
