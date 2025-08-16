use anyhow::Result;
use sage_core::CliOutput;
use sage_core::workflows::list_branches::list_branches_with_output;

pub fn list(relative: bool, global_config: &crate::GlobalConfig) -> Result<()> {
    let cli_config = sage_core::cli::GlobalConfig::new(global_config.json, global_config.no_color);
    let cli = CliOutput::new(cli_config);
    cli.header("list");

    list_branches_with_output(relative, Some(&cli))?;

    cli.summary();
    Ok(())
}
