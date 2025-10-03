use std::{
    fs,
    io::ErrorKind,
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result};

const FETCH_STAMP_FILE: &str = "sage-last-fetch";
// TODO: make configurable
// FETCH INTERVAL IS HARDCODED TO 5 MINUTES
const FETCH_MAX_AGE_SECS: u64 = 60 * 5;

/// Run `git fetch` when the previous fetch is older than `max_age`.
///
/// Returns `true` when a fetch was performed, `false` when it was skipped because
/// the cached result is still fresh.
pub fn fetch_if_stale(repo: &sage_git::Repo) -> Result<bool> {
    let git_dir = repo.git_dir();
    let stamp_path = git_dir.join(FETCH_STAMP_FILE);

    let now = SystemTime::now();
    let should_fetch = match fs::metadata(&stamp_path) {
        Ok(metadata) => match metadata.modified() {
            Ok(modified) => match now.duration_since(modified) {
                Ok(elapsed) => elapsed >= Duration::from_secs(FETCH_MAX_AGE_SECS),
                Err(_) => false,
            },
            Err(_) => true,
        },
        Err(err) if err.kind() == ErrorKind::NotFound => true,
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to read metadata for {}", stamp_path.display()));
        }
    };

    if !should_fetch {
        return Ok(false);
    }

    repo.fetch()?;

    fs::write(&stamp_path, b"fetched")
        .with_context(|| format!("failed to update fetch stamp at {}", stamp_path.display()))?;

    Ok(true)
}
