use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Command};

mod cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Save(command) => command.run(),
        Command::Work(command) => command.run(),
        Command::List(command) => command.run(),
        Command::Log(command) => command.run(),
    }
}
