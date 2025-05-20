use anyhow::Result;
use sage_core::{ChangeBranchOpts, change_branch};
use sage_git::branch::exists;
use std::time::Instant;

pub fn work(args: &crate::WorkArgs) -> Result<()> {
    println!("🌿  sage — work\n");

    // Starting timer
    let start = Instant::now();

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
    
    change_branch(opts)?;

    println!("\nDone in {:?}", start.elapsed());
    Ok(())
}
