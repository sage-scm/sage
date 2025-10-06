use anyhow::Result;
use clap::Args;
use sage_core::save;

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
    pub async fn run(self) -> Result<()> {
        let console = sage_fmt::Console::new();
        console.header("save")?;

        save(self.message, self.force, self.ai, self.push, &console).await
    }
}
