use crate::prelude::{Git, GitResult};
use anyhow::{Result, bail};

pub fn run_pre_commit() -> GitResult<()> {
    let output = Git::new("hook")
        .args(vec!["run", "pre-commit"])
        .raw_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Pre-commit hooks failed: {}", stderr.trim());
    }

    Ok(())
}
