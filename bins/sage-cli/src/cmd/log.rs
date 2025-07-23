use anyhow::Result;
use sage_core::{CliOutput, log_commits};

pub fn log() -> Result<()> {
    let cli = CliOutput::new();
    cli.header("Log");

    log_commits()?;

    cli.summary();
    Ok(())
}
