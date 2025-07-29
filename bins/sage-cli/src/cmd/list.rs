use anyhow::Result;
use sage_core::CliOutput;
use sage_core::list_branches;

pub fn list(relative: bool) -> Result<()> {
    let cli = CliOutput::new();
    cli.header("list");

    list_branches(relative)?;

    cli.summary();
    Ok(())
}
