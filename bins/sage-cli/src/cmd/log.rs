use anyhow::Result;
use sage_core::{CliOutput, log_commits};

pub fn log(limit: Option<i64>, global_config: &crate::GlobalConfig) -> Result<()> {
    let cli_config = sage_core::cli::GlobalConfig::new(global_config.json, global_config.no_color);
    let cli = CliOutput::new(cli_config);
    cli.header("Log");

    let limit_usize = limit.and_then(|l| if l > 0 { Some(l as usize) } else { None });
    log_commits(limit_usize)?;

    cli.summary();
    Ok(())
}
