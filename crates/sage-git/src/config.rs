use anyhow::Result;
use std::process::Command;

/// Get a git config value
pub fn get_config(key: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(&["config", "--get", key])
        .output()?;

    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    } else {
        Ok(None)
    }
}

/// Check if branch should use rebase based on git config
pub fn should_branch_rebase(branch: &str) -> Result<Option<bool>> {
    // Check branch-specific rebase setting
    if let Some(value) = get_config(&format!("branch.{}.rebase", branch))? {
        return Ok(Some(value == "true"));
    }

    // Check global pull.rebase setting
    if let Some(value) = get_config("pull.rebase")? {
        return Ok(Some(value == "true"));
    }

    Ok(None)
}
