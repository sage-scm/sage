pub mod config;
pub mod list;
pub mod log;
pub mod save;
pub mod start;
pub mod work;

pub use config::ConfigCommand;
pub use list::ListCommand;
pub use log::LogCommand;
pub use save::SaveCommand;
pub use start::StartCommand;
pub use work::WorkCommand;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "sg", version, author, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Start(StartCommand),
    Save(SaveCommand),
    Work(WorkCommand),
    List(ListCommand),
    Log(LogCommand),
    Config(ConfigCommand),
}
