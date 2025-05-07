use anyhow::Result;
use sage_core::{ChangeBranchOpts, change_branch};
use std::time::Instant;

pub fn work(args: &crate::WorkArgs) -> Result<()> {
    println!("🌿  sage — work\n");

    // Starting timer
    let start = Instant::now();

    let opts = ChangeBranchOpts {
        name: args.branch.clone().unwrap_or_default().to_string(),
        create: true,
        fetch: !args.no_fetch,
        use_root: args.root,
        push: args.push,
        fuzzy: args.fuzzy,
    };
    change_branch(opts)?;

    println!("\nDone in {:?}", start.elapsed());
    Ok(())
}
