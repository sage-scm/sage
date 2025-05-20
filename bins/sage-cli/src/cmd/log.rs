use anyhow::Result;
use std::time::Instant;

pub fn log(_args: &crate::LogArgs) -> Result<()> {
    println!("🌿  sage — log\n");

    // Starting timer
    let start = Instant::now();

    

    println!("\nDone in {:?}", start.elapsed());
    Ok(())
}