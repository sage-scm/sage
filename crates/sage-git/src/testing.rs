use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{Context, Result, bail};
use tempfile::TempDir;

use crate::Repo;

/// Builder for [`TestRepo`].
#[derive(Debug, Clone)]
pub struct TestRepoBuilder {
    initial_branch: Option<String>,
    user_name: String,
    user_email: String,
    initial_commit: bool,
}

impl Default for TestRepoBuilder {
    fn default() -> Self {
        Self {
            initial_branch: Some("main".to_owned()),
            user_name: "Test User".to_owned(),
            user_email: "test@example.com".to_owned(),
            initial_commit: false,
        }
    }
}

impl TestRepoBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initial_branch(mut self, name: impl Into<String>) -> Self {
        self.initial_branch = Some(name.into());
        self
    }

    pub fn no_initial_branch(mut self) -> Self {
        self.initial_branch = None;
        self
    }

    pub fn user_name(mut self, name: impl Into<String>) -> Self {
        self.user_name = name.into();
        self
    }

    pub fn user_email(mut self, email: impl Into<String>) -> Self {
        self.user_email = email.into();
        self
    }

    pub fn with_initial_commit(mut self) -> Self {
        self.initial_commit = true;
        self
    }

    pub fn build(self) -> Result<TestRepo> {
        let dir = TempDir::new().context("creating temporary repository directory")?;

        let init_cmd = if let Some(branch) = &self.initial_branch {
            GitCommand::new(dir.path())
                .arg("init")
                .arg("-b")
                .arg(branch)
        } else {
            GitCommand::new(dir.path()).arg("init")
        };
        init_cmd
            .run()
            .context("initializing temporary repository")?;

        GitCommand::new(dir.path())
            .arg("config")
            .arg("user.name")
            .arg(&self.user_name)
            .run()
            .context("configuring user.name in temporary repository")?;
        GitCommand::new(dir.path())
            .arg("config")
            .arg("user.email")
            .arg(&self.user_email)
            .run()
            .context("configuring user.email in temporary repository")?;

        let repo = Repo::discover(dir.path()).context("discovering temporary repository")?;
        let test_repo = TestRepo { dir, repo };

        if self.initial_commit {
            test_repo
                .commit_allow_empty("Initial commit")
                .context("creating initial commit in temporary repository")?;
        }

        Ok(test_repo)
    }
}

/// Utility for tests that need to manipulate a real git repository.
#[derive(Debug)]
pub struct TestRepo {
    dir: TempDir,
    repo: Repo,
}

impl TestRepo {
    pub fn new() -> Result<Self> {
        TestRepoBuilder::default().build()
    }

    pub fn builder() -> TestRepoBuilder {
        TestRepoBuilder::default()
    }

    pub fn repo(&self) -> &Repo {
        &self.repo
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn git(&self) -> GitCommand {
        GitCommand::new(self.path())
    }

    pub fn run_git<I, S>(&self, args: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.git().args(args).run()
    }

    pub fn write(&self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<PathBuf> {
        let path = self.path().join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating parent directories for {}", path.display()))?;
        }
        fs::write(&path, contents).with_context(|| format!("writing file {}", path.display()))?;
        Ok(path)
    }

    pub fn commit_all(&self, message: &str) -> Result<()> {
        self.git().arg("add").arg("--all").run().with_context(|| {
            format!("adding all files before commit \"{message}\" in temporary repository")
        })?;
        self.commit(message)
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        self.git()
            .arg("commit")
            .arg("-m")
            .arg(message)
            .run()
            .with_context(|| format!("creating commit \"{message}\" in temporary repository"))
    }

    pub fn commit_allow_empty(&self, message: &str) -> Result<()> {
        self.git()
            .arg("commit")
            .arg("--allow-empty")
            .arg("-m")
            .arg(message)
            .run()
            .with_context(|| format!("creating empty commit \"{message}\" in temporary repository"))
    }
}

impl std::ops::Deref for TestRepo {
    type Target = Repo;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

/// Thin wrapper over `git` commands that runs them in the [`TestRepo`] directory.
#[derive(Debug)]
pub struct GitCommand {
    command: Command,
}

impl GitCommand {
    fn new(cwd: &Path) -> Self {
        let mut command = Command::new("git");
        command.current_dir(cwd);
        Self { command }
    }

    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.command.arg(arg);
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.command.env(key, value);
        self
    }

    pub fn output(mut self) -> Result<Output> {
        self.command
            .output()
            .context("running git command in temporary repository")
    }

    pub fn run(self) -> Result<()> {
        let output = self.output()?;
        if output.status.success() {
            return Ok(());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "git command failed (status {}):\nstdout:\n{}\nstderr:\n{}",
            output
                .status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "terminated by signal".to_owned()),
            stdout,
            stderr
        );
    }
}
