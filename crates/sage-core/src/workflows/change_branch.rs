use anyhow::Result;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use sage_git::{
    branch::{
        exists, get_current, get_default_branch, is_default, is_default_branch, list_branches, push,
        switch,
    },
    repo::{fetch_remote, get_commiter},
};
use sage_graph::SageGraph;
use std::time::Instant;

#[derive(Debug, Default)]
pub struct ChangeBranchOpts {
    pub name: String,
    /// Can the branch be created?
    pub create: bool,
    /// Should we fetch remote first?
    pub fetch: bool,
    /// Should we switch to root branch first?
    pub use_root: bool,
    /// Push to remote?
    pub push: bool,
    /// Use fuzzy search for branch name?
    pub fuzzy: bool,
}

pub fn change_branch(mut opts: ChangeBranchOpts) -> Result<()> {
    println!("🌿  sage — work");

    // Starting timer
    let start = Instant::now();
    let current_branch = get_current()?;

    // Handle fuzzy search if enabled
    if opts.fuzzy && !opts.name.is_empty() {
        // Skip fuzzy search if the branch exists exactly as specified
        if !exists(&opts.name)? {
            let branches = list_branches()?;

            // Initialize the fuzzy matcher
            let matcher = SkimMatcherV2::default();
            let mut best_match = None;
            let mut best_score = 0;

            // Find the best match using fuzzy-matcher
            for branch in branches {
                if let Some(score) = matcher.fuzzy_match(&branch, &opts.name) {
                    if score > best_score {
                        best_score = score;
                        best_match = Some(branch);
                    }
                }
            }

            // Use the best match if found
            if let Some(branch_name) = best_match {
                println!("🔍 Fuzzy matched '{}' to '{}'", opts.name, branch_name);
                opts.name = branch_name;
            }
        }
    }

    let name = &opts.name;

    if opts.fetch {
        fetch_remote()?;
        println!("●  Fetch remote ✔");
    }

    if *name == current_branch {
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
        push(name, false)?;
        println!("●  Push origin/{name} ✔");
    }

    let mut graph = SageGraph::load_or_default()?;
    if !graph.tracks(name) && !graph.is_loose(name) {
        let (user_name, _) = get_commiter()?;
        graph.add_loose_branch(name.into(), current_branch, user_name)?;
        graph.save()?;
    }

    println!("🚀  Switched to {name}");
    println!("Done in {:?}", start.elapsed());

    Ok(())
}
