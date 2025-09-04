use anyhow::Result;
use sage_core::share_branch;
use sage_tui::Tui;

pub async fn share(args: &crate::ShareArgs, _global_config: &crate::GlobalConfig) -> Result<()> {
    let tui = Tui::new();

    tui.header("share")?;

    share_branch(&tui).await?;

    Ok(())
}
