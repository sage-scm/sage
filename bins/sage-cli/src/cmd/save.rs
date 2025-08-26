use anyhow::Result;
use sage_core::SaveOpts;
use sage_tui::Tui;

pub async fn save(args: &crate::SaveArgs, global_config: &crate::GlobalConfig) -> Result<()> {
    let tui = if global_config.no_color {
        Tui::with_theme(sage_tui::Theme::monochrome())
    } else {
        Tui::new()
    };

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

    let start_time = std::time::Instant::now();
    sage_core::save(&opts, &tui).await?;

    if !global_config.json {
        let elapsed = start_time.elapsed();
        let duration_str = format!("{:.3}s", elapsed.as_secs_f64());
        tui.message(
            sage_tui::MessageType::Success,
            &format!("Done in {}", duration_str),
        )?;
    }

    Ok(())
}
