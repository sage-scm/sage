use anyhow::Result;
use sage_core::CliOutput;

pub fn up() -> Result<()> {
    let cli = CliOutput::new();
    cli.header("Stack - Prev");

    sage_core::workflows::stack_navigate::navigate(true, &cli)?;

    cli.summary();
    Ok(())
}

pub fn down() -> Result<()> {
    let cli = CliOutput::new();
    cli.header("Stack - Next");

    sage_core::workflows::stack_navigate::navigate(false, &cli)?;

    cli.summary();
    Ok(())
}
