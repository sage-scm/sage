use std::{
    ffi::OsString,
    path::PathBuf,
    process::{Command, ExitStatus, Output, Stdio},
};

use anyhow::{Result, anyhow, bail};

use crate::Repo;

/// Helper to build and run git commands scoped to a repository.
#[derive(Debug, Default)]
pub struct GitCommand {
    repo_path: PathBuf,
    args: Vec<OsString>,
    env: Vec<(OsString, OsString)>,
    env_remove: Vec<OsString>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    allow_failure: bool,
}

impl GitCommand {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
            ..Default::default()
        }
    }

    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub fn env_remove(mut self, key: impl Into<OsString>) -> Self {
        self.env_remove.push(key.into());
        self
    }

    pub fn stdin(mut self, stdin: Stdio) -> Self {
        self.stdin = Some(stdin);
        self
    }

    pub fn stdout(mut self, stdout: Stdio) -> Self {
        self.stdout = Some(stdout);
        self
    }

    pub fn stderr(mut self, stderr: Stdio) -> Self {
        self.stderr = Some(stderr);
        self
    }

    pub fn allow_failure(mut self) -> Self {
        self.allow_failure = true;
        self
    }

    pub fn require_success(mut self) -> Self {
        self.allow_failure = false;
        self
    }

    pub fn run(self) -> Result<()> {
        self.run_with_status().map(|_| ())
    }

    pub fn run_with_status(mut self) -> Result<ExitStatus> {
        let command_line = self.command_line();
        let status = self.prepare_command().status()?;
        if !self.allow_failure && !status.success() {
            bail!("Git command failed: {command_line}");
        }
        Ok(status)
    }

    pub fn run_with_output(mut self) -> Result<Output> {
        let command_line = self.command_line();
        let output = self.prepare_command().output()?;
        if !self.allow_failure && !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr_trimmed = stderr.trim();
            if stderr_trimmed.is_empty() {
                bail!("Git command failed: {command_line}");
            } else {
                bail!("Git command failed: {command_line}: {stderr_trimmed}");
            }
        }
        Ok(output)
    }

    fn command_line(&self) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 3);
        parts.push("git".to_string());
        parts.push("-C".to_string());
        parts.push(self.repo_path.display().to_string());
        parts.extend(
            self.args
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned()),
        );
        parts.join(" ")
    }

    fn prepare_command(&mut self) -> Command {
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.repo_path);

        // Align with git-butler defaults to reduce lock contention, disable pager, and
        // opt into modern protocol negotiations. Users can still override them via
        // explicit args added afterwards.
        command.arg("-c").arg("protocol.version=2");
        command.arg("--no-optional-locks");
        command.arg("--no-pager");

        command.args(&self.args);

        // Prefer non-interactive, consistent output for downstream parsing.
        command.env("GIT_TERMINAL_PROMPT", "0");
        command.env("LC_ALL", "C");

        for (key, value) in &self.env {
            command.env(key, value);
        }

        for key in &self.env_remove {
            command.env_remove(key);
        }

        if let Some(stdin) = self.stdin.take() {
            command.stdin(stdin);
        }

        if let Some(stdout) = self.stdout.take() {
            command.stdout(stdout);
        }

        if let Some(stderr) = self.stderr.take() {
            command.stderr(stderr);
        }

        command
    }
}

impl Repo {
    pub(crate) fn git(&self) -> Result<GitCommand> {
        let workdir = self
            .repo
            .workdir()
            .ok_or_else(|| anyhow!("repository has no worktree"))?;
        Ok(GitCommand::new(workdir))
    }
}
