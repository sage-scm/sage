use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::Repo;

enum NormalizedPath {
    Root,
    Path(PathBuf),
}

impl Repo {
    pub fn stage_all(&self) -> Result<()> {
        self.git()?.arg("add").arg("--all").run()
    }

    pub fn stage_paths<I, P>(&self, paths: I) -> Result<()>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut command = self.git()?.arg("add");
        let mut has_paths = false;

        for path in paths {
            match self.normalize_path(path.as_ref())? {
                NormalizedPath::Root => {
                    return self.stage_all();
                }
                NormalizedPath::Path(rela) => {
                    command = command.arg(rela);
                    has_paths = true;
                }
            }
        }

        if !has_paths {
            return Ok(());
        }

        command.run()
    }

    fn normalize_path(&self, path: &Path) -> Result<NormalizedPath> {
        let workdir = self
            .repo
            .workdir()
            .ok_or_else(|| anyhow!("repository has no worktree"))?;
        let current_dir = self.repo.current_dir();

        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            current_dir.join(path)
        };

        let relative = absolute.strip_prefix(workdir).with_context(|| {
            format!(
                "path '{}' is outside the repository worktree",
                absolute.display()
            )
        })?;

        if relative.as_os_str().is_empty() {
            return Ok(NormalizedPath::Root);
        }

        Ok(NormalizedPath::Path(relative.to_path_buf()))
    }
}
