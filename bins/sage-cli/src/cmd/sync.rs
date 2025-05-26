use anyhow::Result;
use sage_core::workflows::sync_branch;
use sage_core::CliOutput;

/// Synchronize the current branch with the remote.
pub fn sync(_args: &crate::SyncArgs) -> Result<()> {
    let cli = CliOutput::new();
    cli.header("sync");

    sync_branch(&cli)?;

    cli.summary();
    Ok(())
}
