use anyhow::Result;
use sage_config::Config;

pub fn remove_config(key: String) -> Result<()> {
    let mut config = Config::load()?;
    config.remove(&key)?;

    Ok(())
}
