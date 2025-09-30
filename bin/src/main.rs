use anyhow::Result;
use clap::Parser;

mod cli;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Command::Save(command) => command.run(),
        cli::Command::Work(command) => command.run(),
    }
}
