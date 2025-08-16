use crate::prelude::git_ok;
use anyhow::Result;

pub struct AmendOpts {
    // The message to amend with
    pub message: String,
    // Create an empty git commit
    pub empty: bool,
    // Edit without modifying the message
    pub no_edit: bool,
}

// Amend the previous commit with the given message
pub fn amend(opts: &AmendOpts) -> Result<()> {
    let mut args = vec!["commit", "--amend"];

    if opts.empty {
        args.push("--allow-empty");
    }

    if opts.no_edit || opts.message.is_empty() {
        args.push("--no-edit");
    }

    if !opts.message.is_empty() && !opts.empty {
        args.push("-m");
        args.push(&opts.message);
    }

    git_ok(args)
}
