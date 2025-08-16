use anyhow::Result;
use sage_core::{BranchName, CliOutput};

pub fn stack_init(stack_name: &str, global_config: &crate::GlobalConfig) -> Result<()> {
    let cli_config = sage_core::cli::GlobalConfig::new(global_config.json, global_config.no_color);
    let cli = CliOutput::new(cli_config);
    cli.header("Stack");

    let branch_name = BranchName::new(stack_name)?;

    sage_core::stack_init(branch_name, &cli)?;

    cli.summary();
    Ok(())
}
