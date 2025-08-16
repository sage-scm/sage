use anyhow::Result;
use sage_core::{CliOutput, SaveOpts};

pub async fn save(args: &crate::SaveArgs, global_config: &crate::GlobalConfig) -> Result<()> {
    let cli_config = sage_core::cli::GlobalConfig::new(global_config.json, global_config.no_color);
    let cli = CliOutput::new(cli_config);
    cli.header("save");

    let opts = SaveOpts {
        message: args.message.clone().unwrap_or_default(),
        all: args.all,
        paths: args.paths.clone().unwrap_or_default(),
        ai: args.ai,
        amend: args.amend,
        push: args.push || args.amend,
        empty: args.empty,
    };
    sage_core::save(&opts, &cli).await?;

    cli.summary();
    Ok(())
}
