use anyhow::Result;
use sage_core::{log_commits, CliOutput};

pub fn log() -> Result<()> {
    let cli = CliOutput::new();
    cli.header("Log");

    log_commits(&cli)?;

    cli.summary();
    Ok(())
}
