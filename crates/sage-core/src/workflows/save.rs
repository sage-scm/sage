use std::time::Instant;

use anyhow::Result;
use sage_git::branch::is_clean;
use sage_utils::defer;

#[derive(Debug, Default)]
pub struct SaveOpts {
    /// The message to commit with
    pub message: Option<String>,
    /// Commit all changes
    pub all: bool,
    /// Commit only these paths
    pub paths: Option<Vec<String>>,
    /// Use AI to generate a commit message
    pub ai: bool,
    /// Amend the previous commit
    pub amend: bool,
    /// Push to remote
    pub push: bool,
}

pub fn save(opts: &SaveOpts) -> Result<()> {
    println!("🌿  sage — save");
    let start = Instant::now();
    defer! {
        println!("Done in {:?} s", start.elapsed());
    };
    // First we will check if there are any changes to commit.
    // If there are none, we will simply return early.
    if is_clean()? {
        return Ok(());
    }

    Ok(())
}
