use crate::prelude::{Git, GitResult};

pub fn get_config(key: &str) -> GitResult<Option<String>> {
    let output = Git::new("config").args(["--get", key]).raw_output()?;

    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    } else {
        Ok(None)
    }
}

pub fn should_branch_rebase(branch: &str) -> GitResult<Option<bool>> {
    if let Some(value) = get_config(&format!("branch.{branch}.rebase"))? {
        return Ok(Some(value == "true"));
    }

    if let Some(value) = get_config("pull.rebase")? {
        return Ok(Some(value == "true"));
    }

    Ok(None)
}
