use anyhow::Result;
use sage_core::{change_branch, ChangeBranchOpts};

pub fn work(name: &str) -> Result<()> {
    let opts = ChangeBranchOpts {
        create: true,
        fetch: true,
        use_root: true,
        push: true,
    };
    change_branch(name, opts)
}
