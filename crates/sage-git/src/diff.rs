use anyhow::{Result, anyhow};
use std::process::Command;

/// Returns a unified diff (patch) of staged changes, suitable for AI commit message generation.
/// Output includes context for understanding what was changed and where.
///
pub fn diff_ai() -> Result<String> {
    // Get the staged changes as a unified diff
    let output = Command::new("git")
        .args(["diff", "--staged", "-U10"]) // -U10 for more context, adjust as needed
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!("Failed to get git diff: {}", err));
    }

    let diff = String::from_utf8(output.stdout)?.trim().to_string();

    if diff.is_empty() {
        return Err(anyhow!("No staged changes to diff."));
    }

    // Optionally, add a short summary/statistics to the beginning for context
    let stat_output = Command::new("git")
        .args(["diff", "--staged", "--stat"])
        .output()?;

    let stat = String::from_utf8(stat_output.stdout)?.trim().to_string();
    let summary = if !stat.is_empty() {
        format!("# Staged changes summary (stat):\n{stat}\n\n# Diff Content\n")
    } else {
        "# Diff Content\n".to_string()
    };

    Ok(format!("{summary}{diff}"))
}
