use crate::prelude::{Git, GitResult};

pub struct AmendOpts {
    pub message: String,
    pub empty: bool,
    pub no_edit: bool,
}

pub fn amend(opts: &AmendOpts) -> GitResult<()> {
    let mut git = Git::new("commit").arg("--amend");

    if opts.empty {
        git = git.arg("--allow-empty");
    }

    if opts.no_edit || opts.message.is_empty() {
        git = git.arg("--no-edit");
    }

    if !opts.message.is_empty() && !opts.empty {
        git = git.args(["-m", &opts.message]);
    }

    git.context("Failed to amend commit").run()
}
