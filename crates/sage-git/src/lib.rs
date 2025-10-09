use std::path::{Path, PathBuf};

use anyhow::Result;
use gix::{Repository, bstr::ByteSlice};

mod branch;
mod commit;
mod config;
mod diff;
mod exec;
mod stage;
mod status;

#[derive(Debug)]
pub struct Repo {
    /// Repo
    repo: Repository,
}

impl Repo {
    pub fn discover(path: impl AsRef<Path>) -> Result<Self> {
        let mut repo = gix::discover(path)?;
        // Set object cache if not already set (16 MB _*should*_ be safe enough)
        repo.object_cache_size_if_unset(16 * 1024 * 1024);
        Ok(Self { repo })
    }

    pub fn open() -> Result<Self> {
        Self::discover(".")
    }

    pub fn git_dir(&self) -> PathBuf {
        self.repo.path().to_path_buf()
    }

    pub fn workdir(&self) -> Option<PathBuf> {
        self.repo.workdir().map(|dir| dir.to_path_buf())
    }

    pub fn repo_root(&self) -> PathBuf {
        self.workdir().unwrap_or_else(|| self.git_dir())
    }

    pub fn author_name(&self) -> Result<Option<String>> {
        let signature = match self.repo.author() {
            Some(result) => result?,
            None => return Ok(None),
        };
        Ok(Some(signature.name.to_str_lossy().into_owned()))
    }
}
