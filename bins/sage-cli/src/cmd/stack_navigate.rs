use anyhow::Result;
use sage_core::CliOutput;

pub fn up(global_config: &crate::GlobalConfig) -> Result<()> {
    let cli_config = sage_core::cli::GlobalConfig::new(global_config.json, global_config.no_color);
    let cli = CliOutput::new(cli_config);
    cli.header("Stack - Prev");

    sage_core::workflows::stack_navigate::navigate(true, &cli)?;

    cli.summary();
    Ok(())
}

pub fn down(global_config: &crate::GlobalConfig) -> Result<()> {
    let cli_config = sage_core::cli::GlobalConfig::new(global_config.json, global_config.no_color);
    let cli = CliOutput::new(cli_config);
    cli.header("Stack - Next");

    sage_core::workflows::stack_navigate::navigate(false, &cli)?;

    cli.summary();
    Ok(())
}
