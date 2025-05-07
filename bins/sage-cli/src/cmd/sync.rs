use anyhow::Result;
use sage_core::workflows::sync_branch;

/// Synchronize the current branch with the remote.
pub fn sync(_args: &crate::SyncArgs) -> Result<()> {
    sync_branch()
}
