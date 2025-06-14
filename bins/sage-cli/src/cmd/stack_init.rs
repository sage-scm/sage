use anyhow::Result;
use sage_core::{BranchName, CliOutput};

pub fn stack_init(stack_name: &str) -> Result<()> {
    let cli = CliOutput::new();
    cli.header("Stack");

    let branch_name = BranchName::new(stack_name)?;

    sage_core::stack_init(branch_name, &cli)?;

    cli.summary();
    Ok(())
}
