use anyhow::Result;
use gix::Repository;

mod branch;
mod commit;
mod config;
mod exec;
mod status;

#[derive(Debug)]
pub struct Repo {
    /// Repo
    repo: Repository,
}

impl Repo {
    pub fn open() -> Result<Self> {
        let repo = gix::discover(".")?;
        Ok(Self { repo })
    }
}
