use anyhow::Result;
use sage_core::SaveOpts;

pub async fn save(args: &crate::SaveArgs) -> Result<()> {
    let opts = SaveOpts {
        message: args.message.clone(),
        all: args.all,
        paths: args.paths.clone(),
        ai: args.ai,
        amend: args.amend,
        push: args.push,
        empty: args.empty,
    };
    sage_core::save(&opts).await
}
