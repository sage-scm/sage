use anyhow::bail;
use sage_git::branch::{is_clean, is_merge_in_progress};
use sage_git::rebase::is_rebase_in_progress;
use sage_git::stash::stash_push;
use crate::{CliOutput, SyncBranchOpts};

pub fn stash_if_dirty(cli: &CliOutput, opts: &SyncBranchOpts) -> anyhow::Result<bool> {
    if is_clean()? {
        return Ok(false);
    }

    if is_rebase_in_progress()? {
        bail!("Rebase in progress. Please complete it with 'sage sync --continue' or abort with 'sage sync --abort'");
    }

    if is_merge_in_progress()? {
        bail!("Merge in progress. Please complete it with 'sage sync --continue' or abort with 'sage sync --abort'");
    }

    if !opts.autostash {
        bail!("Working directory is not clean. Please commit or stash your changes before syncing, or use --autostash.");
    }

    cli.step_start("Stashing local changes");
    stash_push(Some("sage sync autostash"))?;
    cli.step_success("Changes stashed", Some("will restore after sync"));
    Ok(true)
}
