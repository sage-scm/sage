use anyhow::Result;
use sage_git::branch::{exists, switch};
use std::time::Instant;

pub fn create_branch(name: &str) -> Result<()> {
    println!("🌿  sage — work");
    let start = Instant::now();
    // Let's see if the new brach already exists.
    if exists(name)? {
        println!("  Branch exists - checking it out.");
        // We will simply switch to the branch.
        switch(name, false)?;
    } else {
        // We will create the branch.
        switch(name, true)?;
    }
    println!("🚀 Switched to {name}");
    println!("Done in {:?} s", start.elapsed());

    Ok(())
}
