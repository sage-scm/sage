use std::time::Instant;

use anyhow::Result;
use sage_core::workflows::sync_branch;

/// Synchronize the current branch with the remote.
pub fn sync(_args: &crate::SyncArgs) -> Result<()> {

    println!("🌿  sage — sync");

    // Starting timer
    let start = Instant::now();

    sync_branch()?;

    println!("\nDone in {:?}", start.elapsed());
    Ok(())
}
