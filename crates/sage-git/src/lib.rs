use std::path::PathBuf;

use anyhow::Result;
use gix::Repository;

mod branch;
mod commit;
mod config;
mod exec;
mod stage;
mod status;

#[derive(Debug)]
pub struct Repo {
    /// Repo
    repo: Repository,
}

impl Repo {
    pub fn open() -> Result<Self> {
        let mut repo = gix::discover(".")?;
        // Set object cache if not already set (16 MB _*should*_ be safe enough)
        repo.object_cache_size_if_unset(16 * 1024 * 1024);
        Ok(Self { repo })
    }

    pub fn git_dir(&self) -> PathBuf {
        self.repo.path().to_path_buf()
    }
}
