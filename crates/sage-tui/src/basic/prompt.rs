use anyhow::{bail, Result};
use std::io::{self, Write};
use std::process::Command;
use tempfile::NamedTempFile;

pub fn confirm_yes_no(prompt: &str) -> Result<bool> {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(matches!(
        buf.trim(),
        "y" | "Y" | "yes" | "YES" | "yep" | "YEP"
    ))
}

pub fn prompt_line(prompt: &str) -> Result<String> {
    print!("{} ", prompt);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

pub fn prompt_editor(default: &str) -> Result<String> {
    let config_manager = sage_config::ConfigManager::new()?;
    let config = config_manager.load()?;
    
    let editor = std::env::var("EDITOR")
        .unwrap_or_else(|_| config.editor.clone());
    
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(default.as_bytes())?;
    temp_file.flush()?;
    
    let temp_path = temp_file.path();
    
    let status = if editor == "vi" || editor == "vim" || editor == "nvim" {
        Command::new(&editor)
            .arg(temp_path)
            .status()?
    } else if editor == "code" {
        Command::new(&editor)
            .arg("--wait")
            .arg(temp_path)
            .status()?
    } else if editor == "nano" || editor == "emacs" {
        Command::new(&editor)
            .arg(temp_path)
            .status()?
    } else {
        Command::new(&editor)
            .arg(temp_path)
            .status()?
    };
    
    if !status.success() {
        bail!("Editor exited with non-zero status");
    }
    
    let content = std::fs::read_to_string(temp_path)?;
    Ok(content)
}
