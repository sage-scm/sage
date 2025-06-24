use anyhow::Result;

#[cfg(feature = "tui")]
use sage_tui::display;

pub fn ui() -> Result<()> {
    #[cfg(feature = "tui")]
    {
        display::SageUI::default().run()
    }
    #[cfg(not(feature = "tui"))]
    {
        eprintln!("TUI feature not enabled. Rebuild with --features tui");
        Ok(())
    }
}
