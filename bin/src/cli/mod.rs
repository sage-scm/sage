pub mod list;
pub mod log;
pub mod save;
pub mod work;

pub use list::ListCommand;
pub use log::LogCommand;
pub use save::SaveCommand;
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
    Save(SaveCommand),
    Work(WorkCommand),
    List(ListCommand),
    Log(LogCommand),
}
