use anyhow::Result;
use sage_core::list_branches;
use sage_core::CliOutput;

pub fn list(args: &crate::ListArgs) -> Result<()> {
    let cli = CliOutput::new();
    cli.header("list");

    list_branches(args.stats)?;

    // Create a simple hello world debug.
    // debug!()
    cli.summary();
    Ok(())
}
