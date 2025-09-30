use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct SaveCommand {
    #[arg(short = 'm', long = "message", value_name = "MESSAGE")]
    pub message: Option<String>,
    #[arg(short = 'f', long = "force")]
    pub force: bool,
    #[arg(short = 'a', long = "ai")]
    pub ai: bool,
    #[arg(short = 'p', long = "push")]
    pub push: bool,
}

impl SaveCommand {
    pub fn run(self) -> Result<()> {todo!()}
}
