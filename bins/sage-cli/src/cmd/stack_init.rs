use anyhow::Result;
use sage_core::CliOutput;

pub fn stack_init(stack_name: &str) -> Result<()> {
    let cli = CliOutput::new();
    cli.header("Stack");

    sage_core::stack_init(stack_name, &cli)?;

    cli.summary();
    Ok(())
}
