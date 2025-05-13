use anyhow::Result;
use sage_core::SaveOpts;
use std::time::Instant;

pub async fn save(args: &crate::SaveArgs) -> Result<()> {
    println!("🌿  sage — save\n");
    let start = Instant::now();

    let opts = SaveOpts {
        message: args.message.clone().unwrap_or_default(),
        all: args.all,
        paths: args.paths.clone().unwrap_or_default(),
        ai: args.ai,
        amend: args.amend,
        push: args.push || args.amend,
        empty: args.empty,
    };
    sage_core::save(&opts).await?;

    println!("\nDone in {:?}", start.elapsed());
    Ok(())
}
