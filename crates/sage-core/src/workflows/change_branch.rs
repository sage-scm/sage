use anyhow::Result;
use colored::Colorize;
use sage_git::{
    branch::{
        exists, get_current, get_default_branch, is_default, is_default_branch, push, set_upstream,
        switch,
    },
    repo::fetch_remote,
};
use std::time::Instant;

#[derive(Debug, Default)]
pub struct ChangeBranchOpts {
    /// Can the branch be created?
    pub create: bool,
    /// Should we fetch remote first?
    pub fetch: bool,
    /// Should we switch to root branch first?
    pub use_root: bool,
    /// Push to remote?
    pub push: bool,
}

pub fn change_branch(name: &str, opts: ChangeBranchOpts) -> Result<()> {
    println!("🌿  sage — work");
    // Starting timer
    let start = Instant::now();

    if opts.fetch {
        fetch_remote()?;
        println!("●  Fetch remote ✔");
    }

    if name == get_current()? {
        println!("⚠️  Already on {name}");
        println!("Done in {:?}", start.elapsed());
        return Ok(());
    }

    // Early exit if they are switching to the default branch.
    if !is_default_branch()? && is_default(name)? {
        switch(name, false)?;
        println!("🚀  Switched to {name}");
        println!("Done in {:?}", start.elapsed());
        return Ok(());
    }

    if opts.use_root && !is_default_branch()? {
        switch(&get_default_branch()?, false)?;
        println!("●  Switch to root branch ✔");
    }

    // Let's see if the new brach already exists.
    if exists(name)? {
        println!("⚠️  Branch exists - checking it out.");
        // We will simply switch to the branch.
        switch(name, false)?;
    } else if opts.create {
        // We will create the branch.
        switch(name, true)?;
        println!("●  Create branch {name} ✔");
    }

    if opts.push {
        set_upstream(name)?;
        push(name, false)?;
        println!("●  Push origin/{name} ✔");
    }

    println!("🚀  Switched to {}", name.blue());
    println!("Done in {:?}", start.elapsed());

    Ok(())
}
