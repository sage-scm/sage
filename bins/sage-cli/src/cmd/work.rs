use anyhow::Result;
use sage_core::workflows::create_branch;

pub fn work(name: &str) -> Result<()> {
    create_branch(name)
}
