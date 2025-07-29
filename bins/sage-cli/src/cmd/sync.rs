use anyhow::Result;
use sage_core::{CliOutput, SyncBranchOpts, workflows::sync_branch};

/// Synchronize the current branch with the remote.
pub fn sync(args: &crate::SyncArgs) -> Result<()> {
    let cli = CliOutput::new();
    cli.header("sync");

    // sync_branch(&cli, &SyncBranchOpts::default())?;
    sync_branch(
        &cli,
        &SyncBranchOpts {
            continue_: args.continue_,
            abort: args.abort,
            rebase: args.rebase,
            force_push: args.force,
            parent: args.parent.clone(),
            autostash: args.autostash,
            no_stack: args.no_stack,
        },
    )?;

    cli.summary();
    Ok(())
}
