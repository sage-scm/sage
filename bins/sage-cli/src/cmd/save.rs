use anyhow::Result;

pub async fn save(args: &crate::SaveArgs) -> Result<()> {
    let ui = sage_core::Ui::new(false, false);
    sage_core::get_repo_info(&ui)?;

    let args = sage_core::SaveArgs {
        message: args.message.clone(),
        include: args.paths.clone().unwrap_or_default(),
        ai: args.ai,
        amend: args.amend,
        push: args.push || args.amend,
        empty: args.empty,
        json: false,
        no_color: false,
    };

    sage_core::save(&ui, &args).await?;

    Ok(())
}
