use anyhow::Result;
use sage_core::{BranchName, CliOutput};

pub fn stack_adopt(parent: &str, global_config: &crate::GlobalConfig) -> Result<()> {
    let cli_config = sage_core::cli::GlobalConfig::new(global_config.json, global_config.no_color);
    let cli = CliOutput::new(cli_config);
    cli.header("Adopt");

    let parent_name = BranchName::new(parent)?;
    sage_core::stack_adopt(parent_name, &cli)?;

    cli.summary();
    Ok(())
}
