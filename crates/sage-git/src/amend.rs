use anyhow::{anyhow, Result};
use std::process::Command;

pub struct AmendOpts {
    // The message to amend with
    pub message: String,
    // Create an empty git commit
    pub empty: bool,
    // Edit without modifying the message
    pub no_edit: bool,
}

// Amend the previous commit with the given message
pub fn amend(opts: &AmendOpts) -> Result<()> {
    let mut cmd = Command::new("git");

    cmd.arg("commit");
    cmd.arg("--amend");

    if opts.empty {
        cmd.arg("--allow-empty");
    }

    if opts.no_edit {
        cmd.arg("--no-edit");
    }

    if !opts.message.is_empty() && !opts.empty {
        cmd.arg("-m");
        cmd.arg(&opts.message);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to amend"));
    }

    Ok(())
}
