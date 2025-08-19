use crate::prelude::{Git, GitResult};
use anyhow::anyhow;

pub fn diff_ai() -> GitResult<String> {
    let output = Git::new("diff").args(["--staged", "-U10"]).raw_output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!("Failed to get git diff: {}", err));
    }

    let diff = String::from_utf8(output.stdout)?.trim().to_string();

    if diff.is_empty() {
        return Err(anyhow!("No staged changes to diff."));
    }

    let stat_output = Git::new("diff").args(["--staged", "--stat"]).raw_output()?;

    let stat = String::from_utf8(stat_output.stdout)?.trim().to_string();
    let summary = if !stat.is_empty() {
        format!("# Staged changes summary (stat):\n{stat}\n\n# Diff Content\n")
    } else {
        "# Diff Content\n".to_string()
    };

    Ok(format!("{summary}{diff}"))
}
