use anyhow::Result;
use sage_core::CliOutput;
use sage_core::list_branches;

pub fn list() -> Result<()> {
    let cli = CliOutput::new();
    cli.header("list");

    list_branches()?;

    cli.summary();
    Ok(())
}
