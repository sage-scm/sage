use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct WorkCommand {
    #[arg(value_name = "BRANCH")]
    pub branch: String,
    #[arg(long = "parent", value_name = "PARENT")]
    pub parent: Option<String>,
    #[arg(short = 'z', long = "fuzzy")]
    pub fuzzy: bool,
    #[arg(short = 'p', long = "push")]
    pub push: bool,
    #[arg(short = 'r', long = "root")]
    pub root: bool,
}

impl WorkCommand {
    pub fn run(self) -> Result<()> {
        todo!()
    }
}
