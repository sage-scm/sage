use std::fs::File;

use anyhow::{Result, anyhow, bail};
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
    pub fn fetch(&self) -> Result<()> {
        if !self.has_remote()? {
            return Ok(());
        }

        self.git()?
            .arg("fetch")
            .arg("--no-progress")
            .arg("--all")
            .arg("--prune")
            .run()
    }

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

    pub fn create_branch_from(&self, name: &str, from: &str) -> Result<()> {
        let source_ref = self.as_ref(from);
        let mut old_ref = self.repo.find_reference(&source_ref)?;
        let target_id = match old_ref.try_id() {
            Some(id) => id.detach(),
            None => old_ref.follow_to_object()?.detach(),
        };

        let new_ref_name = self.as_ref(name);

        let log_message = format!("create branch {} from {}", name, from);
        self.repo.reference(
            new_ref_name,
            target_id,
            PreviousValue::MustNotExist,
            log_message,
        )?;

        Ok(())
    }

    pub fn switch_branch(&self, name: &str) -> Result<()> {
        // Prefer gix for performance on clean trees. If there are local changes
        // or untracked files, defer to native `git switch` to preserve user data.
        let has_local_changes = self.is_dirty()? || !self.untracked_files()?.is_empty();
        if has_local_changes {
            let branch_name = self.remove_ref(name);
            return self.git()?.arg("switch").arg(branch_name).run();
        }

        let branch_ref = self.as_ref(name);
        let branch_full = FullName::try_from(branch_ref.as_str())?;

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

    pub fn push(&self, force: bool) -> Result<()> {
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

        let force_flag = if force {
            "--force"
        } else {
            "--force-with-lease"
        };

        // Run external git push (since gix lacks native push)
        self.git()?
            .arg("push")
            .arg(force_flag)
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
        if !self.has_remote()? {
            return Ok(String::from("main"));
        }
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

    pub fn pull(&self) -> Result<()> {
        if !self.has_remote()? {
            return Ok(());
        }

        self.git()?
            .arg("pull")
            .arg("--ff-only")
            .arg("--no-progress")
            .run()
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

#[cfg(test)]
mod tests {
    use crate::testing::TestRepo;

    #[test]
    fn create_branch_adds_reference() {
        let repo = TestRepo::builder()
            .with_initial_commit()
            .build()
            .expect("temp repo");

        repo.create_branch("feature").expect("create branch");

        let branches = repo.list_branches().expect("list branches");
        assert!(
            branches.iter().any(|b| b == "refs/heads/feature"),
            "branches missing new ref: {branches:?}"
        );
    }

    #[test]
    fn switch_branch_updates_head() {
        let repo = TestRepo::builder()
            .with_initial_commit()
            .build()
            .expect("temp repo");
        repo.create_branch("feature").expect("create branch");

        repo.switch_branch("feature").expect("switch branch");

        let current = repo.get_current_branch().expect("current branch");
        assert_eq!(current, "feature");
    }

    #[test]
    fn has_branch_accepts_short_names() {
        let repo = TestRepo::builder()
            .with_initial_commit()
            .build()
            .expect("temp repo");
        repo.create_branch("feature").expect("create branch");

        assert!(
            repo.has_branch("feature".to_owned())
                .expect("check branch exists")
        );
        assert!(
            repo.has_branch("refs/heads/feature".to_owned())
                .expect("check branch exists by ref")
        );
    }

    #[test]
    fn ref_helpers_strip_prefixes() {
        let repo = TestRepo::new().expect("temp repo");
        assert_eq!(repo.remove_ref("refs/heads/main"), "main");
        assert_eq!(repo.remove_ref("refs/remotes/origin/main"), "origin/main");
        assert_eq!(repo.as_ref("main"), "refs/heads/main");
        assert_eq!(repo.as_ref("refs/heads/feature"), "refs/heads/feature");
    }
}
