use anyhow::Result;
use sage_config::Config;

pub fn set_config(key: String, value: String) -> Result<()> {
    let mut config = Config::load()?;

    config.set(&key, &value)?;

    Ok(())
}
