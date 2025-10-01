use std::{
    collections::HashMap,
    fs::{self, File},
    io::ErrorKind,
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow, bail};
use gix::{
    bstr::{BStr, ByteSlice},
    interrupt,
    progress::Discard,
    refs::{
        FullName, Target, TargetRef,
        transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog},
    },
    worktree::stack::state::attributes::Source as AttrSource,
};
use gix_worktree_state::checkout;

use super::Repo;

impl Repo {
    pub fn get_current_branch(&self) -> Result<String> {
        let head_name = self.repo.head_name()?;

        match head_name {
            Some(name) => Ok(name.shorten().to_string()),
            _ => bail!("Detached head"),
        }
    }

    pub fn create_branch(&self, name: &str) -> Result<()> {
        let ref_name = self.as_ref(name);
        let target = self.get_current_commit()?;

        self.repo.reference(
            ref_name,
            target,
            PreviousValue::MustNotExist,
            "created new branch",
        )?;
        Ok(())
    }

    pub fn switch_branch(&self, name: &str) -> Result<()> {
        let branch_ref = self.as_ref(name);
        let branch_full = FullName::try_from(branch_ref.as_str())?;
        let previous_tree_id = self.repo.head_tree_id().ok().map(|id| id.detach());

        let edits = [RefEdit {
            change: Change::Update {
                log: LogChange {
                    mode: RefLog::AndReference,
                    force_create_reflog: false,
                    message: "switch branch".into(),
                },
                expected: PreviousValue::Any,
                new: Target::Symbolic(branch_full),
            },
            name: FullName::try_from("HEAD")?,
            deref: false,
        }];

        self.repo.edit_references(edits)?;

        let work_dir = self
            .repo
            .workdir()
            .ok_or_else(|| anyhow!("repository has no worktree"))?
            .to_path_buf();

        let head_tree_id = self.repo.head_tree_id()?;
        let mut index = self.repo.index_from_tree(&head_tree_id)?;

        let files = Discard;
        let bytes = Discard;
        let mut checkout_options = self.repo.checkout_options(AttrSource::IdMapping)?;
        checkout_options.destination_is_initially_empty = false;

        checkout(
            &mut index,
            work_dir,
            self.repo.objects.clone().into_arc()?,
            &files,
            &bytes,
            &interrupt::IS_INTERRUPTED,
            checkout_options,
        )?;

        index.write(Default::default())?;

        let new_tree_id = head_tree_id.detach();
        self.prune_removed_paths(previous_tree_id, &new_tree_id)?;

        Ok(())
    }

    fn prune_removed_paths(
        &self,
        previous_tree_id: Option<gix::ObjectId>,
        new_tree_id: &gix::ObjectId,
    ) -> Result<()> {
        let Some(previous_tree_id) = previous_tree_id else {
            return Ok(());
        };

        if &previous_tree_id == new_tree_id {
            return Ok(());
        }

        let Some(work_dir) = self.repo.workdir() else {
            return Ok(());
        };
        let work_dir = work_dir.to_path_buf();

        let previous_tree = self.repo.find_tree(previous_tree_id)?;
        let new_tree = self.repo.find_tree(*new_tree_id)?;

        let mut diff_platform = previous_tree
            .changes()
            .context("failed to prepare tree diff while pruning worktree")?;

        let mut files_to_remove: Vec<PathBuf> = Vec::new();
        let mut dirs_to_remove: HashMap<PathBuf, bool> = HashMap::new();

        diff_platform
            .for_each_to_obtain_tree(
                &new_tree,
                |change| -> Result<gix::object::tree::diff::Action> {
                    if let gix::object::tree::diff::Change::Deletion {
                        location,
                        entry_mode,
                        ..
                    } = change
                    {
                        let relative =
                            gix::path::try_from_bstr(location).map_err(anyhow::Error::new)?;
                        let full_path = work_dir.join(relative.as_ref());

                        if entry_mode.is_tree() {
                            dirs_to_remove.entry(full_path).or_insert(false);
                        } else if entry_mode.is_commit() {
                            dirs_to_remove
                                .entry(full_path)
                                .and_modify(|recursive| *recursive = true)
                                .or_insert(true);
                        } else {
                            files_to_remove.push(full_path);
                        }
                    }

                    Ok(gix::object::tree::diff::Action::Continue)
                },
            )
            .context("failed to collect deletions while pruning worktree")?;

        for path in files_to_remove {
            match fs::remove_file(&path) {
                Ok(_) => {}
                Err(err) if err.kind() == ErrorKind::NotFound => {}
                Err(err) if err.kind() == ErrorKind::IsADirectory => {
                    match fs::remove_dir_all(&path) {
                        Ok(_) => {}
                        Err(err) if err.kind() == ErrorKind::NotFound => {}
                        Err(err) => {
                            return Err(err)
                                .context(format!("failed to remove directory {}", path.display()));
                        }
                    }
                }
                Err(err) => {
                    return Err(err).context(format!("failed to remove file {}", path.display()));
                }
            }
        }

        let mut dirs: Vec<(PathBuf, bool)> = dirs_to_remove.into_iter().collect();
        dirs.sort_by(|(a, _), (b, _)| b.components().count().cmp(&a.components().count()));

        for (path, recursive) in dirs {
            if recursive {
                match fs::remove_dir_all(&path) {
                    Ok(_) => {}
                    Err(err) if err.kind() == ErrorKind::NotFound => {}
                    Err(err) => {
                        return Err(err)
                            .context(format!("failed to remove directory {}", path.display()));
                    }
                }
            } else {
                match fs::remove_dir(&path) {
                    Ok(_) => {}
                    Err(err)
                        if matches!(
                            err.kind(),
                            ErrorKind::NotFound | ErrorKind::DirectoryNotEmpty
                        ) => {}
                    Err(err) => {
                        return Err(err)
                            .context(format!("failed to remove directory {}", path.display()));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn list_branches(&self) -> Result<Vec<String>> {
        let mut branches = vec![];
        self.repo
            .references()?
            .local_branches()
            .into_iter()
            .for_each(|branch| {
                for name in branch {
                    branches.push(name.unwrap().name().as_bstr().to_string());
                }
            });
        Ok(branches)
    }

    pub fn list_remote_branches(&self) -> Result<Vec<String>> {
        let mut branches = vec![];
        self.repo
            .references()?
            .remote_branches()
            .into_iter()
            .for_each(|branch| {
                for name in branch {
                    branches.push(name.unwrap().name().as_bstr().to_string());
                }
            });

        Ok(branches)
    }

    pub fn list_all_branches(&self) -> Result<Vec<String>> {
        let mut branches = vec![];

        let local_branches = self.list_branches()?;
        let remote_branches = self.list_remote_branches()?;
        let default_branch = self.get_default_branch()?;

        branches.extend(local_branches);
        branches.extend(remote_branches);
        branches.push(default_branch);

        Ok(branches)
    }

    pub fn has_branch(&self, name: String) -> Result<bool> {
        let branches = self.list_branches()?;
        Ok(branches.contains(&self.as_ref(&name)))
    }

    pub fn set_upstream_named(
        &mut self,
        branch: &str,
        remote: &str,
        remote_branch: &str,
    ) -> Result<()> {
        let config_path = self.repo.git_dir().join("config");
        let merge_ref = self.as_ref(remote_branch);

        let mut config = self.repo.config_snapshot_mut();
        let subsection: Option<&BStr> = Some(branch.as_bytes().as_bstr());

        // Set branch.<current>.remote = remote
        config.set_raw_value_by("branch", subsection, "remote", remote.as_bytes().as_bstr())?;

        // Set branch.<current>.merge = refs/heads/<remote_branch>
        config.set_raw_value_by(
            "branch",
            subsection,
            "merge",
            merge_ref.as_bytes().as_bstr(),
        )?;

        let mut out_file = File::create(&config_path)?;
        config.write_to(&mut out_file)?;

        Ok(())
    }

    pub fn set_upstream(&mut self) -> Result<()> {
        let current_branch = self.get_current_branch()?;
        self.set_upstream_named(&current_branch, "origin", &current_branch)?;
        Ok(())
    }

    pub fn push(&self) -> Result<()> {
        // Get upstream from config using string_by (correct method for subsections)
        let config = self.repo.config_snapshot();
        let current_branch = self.get_current_branch()?;
        let subsection: Option<&BStr> = Some(current_branch.as_bytes().as_bstr());
        let remote_name = config
            .string_by("branch", subsection, "remote")
            .ok_or_else(|| anyhow!("No remote configured for branch {}", current_branch))?
            .to_string();
        let merge = config
            .string_by("branch", subsection, "merge")
            .ok_or_else(|| anyhow!("No merge ref configured for branch {}", current_branch))?
            .to_string();

        // Run external git push (since gix lacks native push)
        self.git()?
            .arg("push")
            .arg("--no-progress")
            .arg(remote_name)
            .arg(format!(
                "{}:{}",
                current_branch,
                merge.strip_prefix("refs/heads/").unwrap_or(&merge)
            ))
            .run()?;

        Ok(())
    }

    pub fn is_detached_head(&self) -> Result<bool> {
        let head = self.repo.head_name()?;
        Ok(head.is_none())
    }

    pub fn remote_name(&self) -> Result<Option<String>> {
        let remote = self.repo.remote_default_name(gix::remote::Direction::Fetch);
        match remote {
            Some(remote) => Ok(Some(remote.to_string())),
            None => Ok(None),
        }
    }

    pub fn has_remote(&self) -> Result<bool> {
        let remote = self.remote_name()?;
        Ok(remote.is_some())
    }

    pub fn get_default_branch(&self) -> Result<String> {
        let remote_name = self.remote_name()?.unwrap();
        let head_ref_name = format!("refs/remotes/{}/{}", remote_name, "HEAD");

        let head_ref = self.repo.find_reference(&head_ref_name)?;

        if let TargetRef::Symbolic(target_name) = head_ref.target() {
            // Extract the last component after '/'
            let branch = target_name.as_bstr();
            let branch = branch
                .rsplitn(2, |&b| b == b'/')
                .next()
                .unwrap_or(&branch[0..]);
            Ok(String::from_utf8_lossy(branch).to_string())
        } else {
            bail!("Unable to determine default branch");
        }
    }

    fn is_ref(&self, name: &str) -> bool {
        name.starts_with("refs/heads/")
    }
    pub fn remove_ref(&self, name: &str) -> String {
        name.replace("refs/heads/", "").replace("refs/remotes/", "")
    }
    pub fn as_ref(&self, name: &str) -> String {
        if self.is_ref(name) {
            name.to_string()
        } else {
            format!("refs/heads/{name}")
        }
    }
}
