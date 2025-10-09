use anyhow::Result;
use clap::Args;
use sage_core::config;

#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[arg(short, long)]
    pub key: Option<String>,
    #[arg(short, long)]
    pub value: Option<String>,
}

impl ConfigCommand {
    pub fn run(self) -> Result<()> {
        let console = sage_fmt::Console::new();
        console.header("config")?;

        config(self.key, self.value, &console)
    }
}
