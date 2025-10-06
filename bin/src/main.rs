use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Command};

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        // Create commits
        Command::Save(command) => command.run().await,
        // Change branches
        Command::Work(command) => command.run(),
        // List branches
        Command::List(command) => command.run(),
        // List commits
        Command::Log(command) => command.run(),
    }
}
