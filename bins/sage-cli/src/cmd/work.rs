use anyhow::Result;
use sage_core::{change_branch, BranchName, ChangeBranchOpts, CliOutput};

pub fn work(args: &crate::WorkArgs) -> Result<()> {
    let cli = CliOutput::new();
    cli.header("work");

    let branch_name = BranchName::new(args.branch.clone().unwrap_or_default())?;

    let opts = ChangeBranchOpts {
        name: branch_name,
        parent: args.parent.clone().unwrap_or_default(),
        create: true,
        fetch: args.fetch,
        use_root: args.root,
        push: args.push,
        fuzzy: args.fuzzy,
        track: true,
        announce: true,
    };

    change_branch(opts, &cli)?;

    cli.summary();
    Ok(())
}
