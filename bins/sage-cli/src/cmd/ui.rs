use anyhow::Result;

// #[cfg(feature = "tui")]
// use sage_tui::display;

pub fn ui(_global_config: &crate::GlobalConfig) -> Result<()> {
    todo!("create tui")
    // #[cfg(feature = "tui")]
    // {
    //     display::SageUI::default().run()
    // }
    // #[cfg(not(feature = "tui"))]
    // {
    //     eprintln!("TUI feature not enabled. Rebuild with --features tui");
    //     Ok(())
    // }
}
