use anyhow::Result;
use gix::Id;

use super::Repo;

impl Repo {
    pub fn get_current_commit(&self) -> Result<Id> {
        let found = self.repo.head_commit()?;
        Ok(found.id())
    }
}
