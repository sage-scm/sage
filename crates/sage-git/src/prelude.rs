use std::process::Command;
use anyhow::{bail, Context, Result};

pub fn git_ok<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let out = run_git(args)?;
    if !out.status.success() {
        bail!("git exited with {}", out.status);
    }
    Ok(())
}

pub fn git_success<I, S>(args: I) -> Result<bool>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let out = run_git(args)?;
    Ok(out.status.success())
}

pub fn git_output<I, S>(args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let out = run_git(args)?;
    if !out.status.success() {
        bail!("git command failed");
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(s)
}

pub fn run_git<I, S>(args: I) -> Result<std::process::Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    let out = Command::new("git").args(&args_vec).output().with_context(|| format!("failed to spawn git {:?}", args_vec))?;
    Ok(out)
}
