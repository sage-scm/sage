use anyhow::Result;
use sage_config::Config;

pub fn list_config() -> Result<Vec<String>> {
    let config = Config::load()?;

    Ok(config.keys())
}
