use anyhow::{Context, Result, bail};
use std::process::{Command, Output};

pub type GitResult<T> = Result<T>;

pub struct Git {
    args: Vec<String>,
    context: Option<String>,
}

impl Git {
    pub fn new(command: &str) -> Self {
        Self {
            args: vec![command.to_string()],
            context: None,
        }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.args
            .extend(args.into_iter().map(|s| s.as_ref().to_string()));
        self
    }

    pub fn context(mut self, ctx: &str) -> Self {
        self.context = Some(ctx.to_string());
        self
    }

    pub fn run(self) -> GitResult<()> {
        let ctx = self
            .context
            .clone()
            .unwrap_or_else(|| format!("git {}", self.args.join(" ")));
        let output = self.execute()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("{}: {}", ctx, stderr.trim());
        }
        Ok(())
    }

    pub fn success(self) -> GitResult<bool> {
        let output = self.execute()?;
        Ok(output.status.success())
    }

    pub fn output(self) -> GitResult<String> {
        let ctx = self
            .context
            .clone()
            .unwrap_or_else(|| format!("git {}", self.args.join(" ")));
        let output = self.execute()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("{}: {}", ctx, stderr.trim());
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn output_lines(self) -> GitResult<Vec<String>> {
        self.output()
            .map(|s| s.lines().map(|line| line.to_string()).collect())
    }

    pub fn raw_output(self) -> GitResult<Output> {
        self.execute()
    }

    fn execute(self) -> GitResult<Output> {
        let ctx = self
            .context
            .clone()
            .unwrap_or_else(|| format!("git {}", self.args.join(" ")));

        Command::new("git")
            .args(&self.args)
            .output()
            .with_context(|| format!("failed to execute: {ctx}"))
    }
}

pub fn git_ok<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    if args_vec.is_empty() {
        bail!("git command requires at least one argument");
    }

    let mut builder = Git::new(&args_vec[0]);
    if args_vec.len() > 1 {
        builder = builder.args(&args_vec[1..]);
    }
    builder.run()
}

pub fn git_success<I, S>(args: I) -> Result<bool>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    if args_vec.is_empty() {
        bail!("git command requires at least one argument");
    }

    let mut builder = Git::new(&args_vec[0]);
    if args_vec.len() > 1 {
        builder = builder.args(&args_vec[1..]);
    }
    builder.success()
}

pub fn git_output<I, S>(args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    if args_vec.is_empty() {
        bail!("git command requires at least one argument");
    }

    let mut builder = Git::new(&args_vec[0]);
    if args_vec.len() > 1 {
        builder = builder.args(&args_vec[1..]);
    }
    builder.output()
}

pub fn run_git<I, S>(args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    if args_vec.is_empty() {
        bail!("git command requires at least one argument");
    }

    let mut builder = Git::new(&args_vec[0]);
    if args_vec.len() > 1 {
        builder = builder.args(&args_vec[1..]);
    }
    builder.raw_output()
}

pub fn parse_branch_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|line| line.trim().trim_start_matches('*').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn parse_null_separated(output: &str) -> Vec<String> {
    output
        .split('\0')
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn is_empty_output(output: &str) -> bool {
    output.trim().is_empty()
}
