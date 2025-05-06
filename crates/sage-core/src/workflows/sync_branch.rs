use anyhow::Result;
use sage_git::branch::get_current;

pub struct SyncBranchOpts {
    pub continue_: bool,
    pub abort: bool,
    pub onto: Option<String>,
}

pub fn sync_branch() -> Result<()> {
    let current_branch = get_current()?;

    Ok(())
}
