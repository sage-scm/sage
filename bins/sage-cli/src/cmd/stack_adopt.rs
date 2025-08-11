use anyhow::Result;
use sage_core::{BranchName, CliOutput};

pub fn stack_adopt(parent: &str) -> Result<()> {
    let cli = CliOutput::new();
    cli.header("Adopt");

    let parent_name = BranchName::new(parent)?;
    sage_core::stack_adopt(parent_name, &cli)?;

    cli.summary();
    Ok(())
}
