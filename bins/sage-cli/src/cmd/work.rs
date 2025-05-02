use anyhow::Result;
use sage_core::{change_branch, ChangeBranchOpts};

pub fn work(name: &str) -> Result<()> {
    change_branch(name, ChangeBranchOpts::default())
}
