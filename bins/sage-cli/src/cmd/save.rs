use anyhow::Result;

pub fn save(args: &crate::SaveArgs) -> Result<()> {
    let mut opts = sage_core::SaveOpts::default();
    if let Some(message) = args.message.clone() {
        opts.message = Some(message);
    }
    opts.all = args.all;
    if let Some(paths) = args.paths.clone() {
        opts.paths = Some(paths);
    }
    opts.ai = args.ai;
    opts.amend = args.amend;
    sage_core::save(&opts)
}
