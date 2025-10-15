use anyhow::Result;
use clap::Parser;
use sage_core::check_for_updates;

use crate::cli::{Cli, Command};

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(err) = check_for_updates().await
        && cfg!(debug_assertions)
    {
        eprintln!("check_for_updates failed: {err:#}");
    }
    let cli = Cli::parse();
    match cli.command {
        // Start a new stack
        Command::Start(command) => command.run(),
        // Create commits
        Command::Save(command) => command.run().await,
        // Change branches
        Command::Work(command) => command.run(),
        // List branches
        Command::List(command) => command.run(),
        // List commits
        Command::Log(command) => command.run(),
        // Manage configuration
        Command::Config(command) => command.run(),
    }
}
