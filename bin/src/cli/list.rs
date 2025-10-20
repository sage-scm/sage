use anyhow::Result;
use clap::Args;

use sage_core::workflows::list_branches;

#[derive(Debug, Args)]
pub struct ListCommand {
    #[arg(long)]
    stack: bool,
}

impl ListCommand {
    pub fn run(self) -> Result<()> {
        let console = sage_fmt::Console::new();
        console.header("list")?;
        list_branches(self.stack)
    }
}
