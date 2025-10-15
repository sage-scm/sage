use anyhow::Result;
use clap::Args;
use sage_core::{StartOptions, start};

#[derive(Debug, Args)]
pub struct StartCommand {
    pub name: String,
    #[arg(long = "parent", value_name = "PARENT")]
    pub parent: Option<String>,
    #[arg(long = "stack", value_name = "STACK")]
    pub stack: Option<String>,
    #[arg(long = "root", short = 'r')]
    pub root: bool,
}

impl StartCommand {
    pub fn run(self) -> Result<()> {
        let console = sage_fmt::Console::new();
        console.header("start")?;

        let options = StartOptions {
            parent: self.parent,
            stack: self.stack,
            name: self.name,
            root: self.root,
        };

        start(&options, &console)
    }
}
