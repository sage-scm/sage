use crate::prelude::{Git, GitResult};
use anyhow::bail;

/// Result of running pre-commit hooks
#[derive(Debug, Clone)]
pub struct HookResult {
    pub success: bool,
    pub output: String,
    pub error_output: String,
}

/// Run pre-commit hooks with enhanced error reporting
pub fn run_pre_commit() -> GitResult<()> {
    let result = run_pre_commit_with_output()?;

    if !result.success {
        bail!("Pre-commit hooks failed: {}", result.error_output.trim());
    }

    Ok(())
}

/// Run pre-commit hooks and return detailed output
pub fn run_pre_commit_with_output() -> GitResult<HookResult> {
    let output = Git::new("hook")
        .args(vec!["run", "pre-commit"])
        .raw_output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(HookResult {
        success: output.status.success(),
        output: stdout.to_string(),
        error_output: stderr.to_string(),
    })
}

/// Check if pre-commit hooks are configured
pub fn has_pre_commit_hooks() -> GitResult<bool> {
    let output = Git::new("hook")
        .args(vec!["list", "pre-commit"])
        .raw_output()?;

    Ok(output.status.success() && !output.stdout.is_empty())
}

/// Run specific hook with enhanced error reporting
pub fn run_hook(hook_name: &str) -> GitResult<HookResult> {
    let output = Git::new("hook").args(vec!["run", hook_name]).raw_output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(HookResult {
        success: output.status.success(),
        output: stdout.to_string(),
        error_output: stderr.to_string(),
    })
}
