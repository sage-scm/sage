use anyhow::Result;
use sage_core::workflows::sync_branch;

pub fn sync(_args: &crate::SyncArgs) -> Result<()> {
    sync_branch()
}
