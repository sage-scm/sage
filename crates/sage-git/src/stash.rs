use crate::prelude::{Git, GitResult};
use anyhow::anyhow;

pub fn stash_push(message: Option<&str>) -> GitResult<()> {
    let mut git = Git::new("stash").arg("push");

    if let Some(msg) = message {
        git = git.args(["-m", msg]);
    }

    let output = git.raw_output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No local changes to save") {
            Ok(())
        } else {
            Err(anyhow!("Failed to stash changes: {}", stderr.trim()))
        }
    }
}

pub fn stash_pop() -> GitResult<()> {
    let output = Git::new("stash").arg("pop").raw_output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No stash entries found") {
            Ok(())
        } else if stderr.contains("CONFLICT") {
            Err(anyhow!(
                "Stash pop resulted in conflicts. Please resolve them manually"
            ))
        } else {
            Err(anyhow!("Failed to pop stash: {}", stderr.trim()))
        }
    }
}

pub fn has_stash() -> GitResult<bool> {
    let output = Git::new("stash").arg("list").raw_output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(!stdout.trim().is_empty())
    } else {
        Err(anyhow!("Failed to check stash status"))
    }
}
