use anyhow::Result;
use gix::config::SnapshotMut;

use super::Repo;

impl Repo {
    pub fn get_config(&mut self) -> Result<SnapshotMut<'_>> {
        Ok(self.repo.config_snapshot_mut())
    }
}
