use anyhow::Result;
use sage_core::{change_branch, BranchName, ChangeBranchOpts, CliOutput};

pub fn work(args: &crate::WorkArgs) -> Result<()> {
    let cli = CliOutput::new();
    cli.header("work");

    let branch_name = BranchName::new(args.branch.clone().unwrap_or_default())?;
    let parent_arg = args.parent.clone().unwrap_or_default();
    let parent_name = if parent_arg.is_empty() {
        String::new()
    } else {
        BranchName::new(parent_arg)?.to_string()
    };

    let opts = ChangeBranchOpts {
        name: branch_name,
        parent: parent_name.to_string(),
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
