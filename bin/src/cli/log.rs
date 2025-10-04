use anyhow::Result;
use clap::Args;
use sage_core::log;

#[derive(Debug, Args)]
pub struct LogCommand {
    /// Limit the number of commits displayed
    #[arg(short, long)]
    pub limit: Option<usize>,
}

impl LogCommand {
    pub fn run(self) -> Result<()> {
        let console = sage_fmt::Console::new();
        console.header("log")?;

        log(self.limit)
    }
}
