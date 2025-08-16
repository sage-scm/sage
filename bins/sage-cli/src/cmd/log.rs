use anyhow::Result;
use sage_core::{CliOutput, log_commits};

pub fn log(global_config: &crate::GlobalConfig) -> Result<()> {
    let cli_config = sage_core::cli::GlobalConfig::new(global_config.json, global_config.no_color);
    let cli = CliOutput::new(cli_config);
    cli.header("Log");

    log_commits()?;

    cli.summary();
    Ok(())
}
