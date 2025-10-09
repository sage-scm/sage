use anyhow::Result;
use clap::Args;

use sage_core::workflows::list_branches;

#[derive(Debug, Args)]
pub struct ListCommand {}

impl ListCommand {
    pub fn run(self) -> Result<()> {
        list_branches()
    }
}
