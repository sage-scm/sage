use anyhow::Result;
use sage_core::{BranchName, ChangeBranchOpts, change_branch};
use sage_tui::Tui;

pub fn work(args: &crate::WorkArgs, global_config: &crate::GlobalConfig) -> Result<()> {
    let tui = if global_config.no_color {
        Tui::with_theme(sage_tui::Theme::monochrome())
    } else {
        Tui::new()
    };

    tui.header("work")?;

    let branch_name = BranchName::new(args.branch.clone().unwrap_or_default())?;
    let parent_arg = args.parent.clone().unwrap_or_default();
    let parent_name = if parent_arg.is_empty() && args.root {
        sage_git::branch::get_default_branch()?
    } else if parent_arg.is_empty() && !args.root {
        // Using the current branch as the parent
        sage_git::branch::get_current()?
    } else {
        BranchName::new(parent_arg)?.to_string()
    };

    let opts = ChangeBranchOpts {
        name: branch_name,
        parent: parent_name,
        create: !args.fuzzy,
        fetch: args.fetch,
        // use_root: args.root,
        push: args.push,
        fuzzy: args.fuzzy,
        track: true,
        // announce: true,
        // json_mode: global_config.json,
    };

    // let start_time = std::time::Instant::now();
    change_branch(opts, &tui)?;

    // if !global_config.json {
    //     let elapsed = start_time.elapsed();
    //     let duration_str = format!("{:.3}s", elapsed.as_secs_f64());
    //     tui.message(
    //         sage_tui::MessageType::Success,
    //         &format!("Done in {}", duration_str),
    //     )?;
    // }

    Ok(())
}
