use anyhow::Result;
use sage_core::{ChangeBranchOpts, change_branch};

pub fn work(args: &crate::WorkArgs) -> Result<()> {
    let opts = ChangeBranchOpts {
        name: args.branch.to_string(),
        create: true,
        fetch: !args.no_fetch,
        use_root: args.root,
        push: args.push,
        fuzzy: args.fuzzy,
    };
    change_branch(opts)
}
