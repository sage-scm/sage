use anyhow::{bail, Result};
use std::{env, process::Command};

pub fn config_edit() -> Result<()> {
    let manager = sage_config::ConfigManager::new()?;
    let path = manager.local_config_path();
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let status = Command::new(editor).arg(path).status()?;
    if !status.success() {
        bail!("Editor exited with error");
    }
    Ok(())
}
