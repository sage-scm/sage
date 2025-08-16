use anyhow::{Result, bail};

pub fn config_unset(key: &str, _global_config: &crate::GlobalConfig) -> Result<()> {
    let manager = sage_config::ConfigManager::new()?;
    let mut cfg = manager.load()?;
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["extras", extra_key] => {
            cfg.extras.remove(*extra_key);
        }
        ["ai", "extras", extra_key] => {
            cfg.ai.extras.remove(*extra_key);
        }
        ["tui", "extras", extra_key] => {
            cfg.tui.extras.remove(*extra_key);
        }
        ["pull_requests", "extras", extra_key] => {
            cfg.pull_requests.extras.remove(*extra_key);
        }
        _ => bail!("Only extras fields can be unset. To reset a field, set it to an empty value."),
    }
    manager.update(&cfg, true)?;
    println!("✔ Saved.");
    Ok(())
}
