use anyhow::Result;
use sage_core::SaveOpts;
use sage_tui::Tui;

pub async fn save(args: &crate::SaveArgs, global_config: &crate::GlobalConfig) -> Result<()> {
    let tui = Tui::new();

    tui.header("save")?;

    let opts = SaveOpts {
        message: args.message.clone().unwrap_or_default(),
        all: args.all,
        paths: args.paths.clone().unwrap_or_default(),
        ai: args.ai,
        amend: args.amend,
        push: args.push || args.amend,
        empty: args.empty,
        json_mode: global_config.json,
    };

    sage_core::save(&opts, &tui).await?;

    Ok(())
}
