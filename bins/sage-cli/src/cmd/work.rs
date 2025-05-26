use anyhow::Result;
use sage_core::{change_branch, ChangeBranchOpts, CliOutput};
use sage_git::branch::exists;

pub fn work(args: &crate::WorkArgs) -> Result<()> {
    let cli = CliOutput::new();
    cli.header("work");

    let branch_name = args.branch.clone().unwrap_or_default().to_string();
    let branch_exists = exists(&branch_name)?;

    // Only fetch if explicitly requested with --fetch and the branch doesn't exist locally
    let should_fetch = args.fetch && !branch_exists;

    let opts = ChangeBranchOpts {
        name: branch_name,
        create: true,
        fetch: should_fetch,
        use_root: args.root,
        push: args.push,
        fuzzy: args.fuzzy,
    };

    change_branch(opts, &cli)?;

    cli.summary();
    Ok(())
}
