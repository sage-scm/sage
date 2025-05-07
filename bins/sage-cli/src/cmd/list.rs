use anyhow::Result;
use sage_core::list_branches;
use std::time::Instant;

pub fn list(args: &crate::ListArgs) -> Result<()> {
    println!("🌿  sage — list\n");
    let start_time = Instant::now();

    list_branches(args.stats)?;

    println!("\nDone in {:?}", start_time.elapsed());
    Ok(())
}
