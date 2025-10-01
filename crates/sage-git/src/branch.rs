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
        FullName, Target, TargetRef, transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog}
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

    pub fn is_detached_head(&self) -> Result<bool> {
        let head = self.repo.head_name()?;
        Ok(head.is_none())
    }

    pub fn remote_name(&self) -> Result<Option<String>> {
        let remote = self.repo.remote_default_name(gix::remote::Direction::Fetch);
        match remote {
            Some(remote) => {
                Ok(Some(remote.to_string()))
            }
            None => {
                Ok(None)
            }
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
            let branch = branch.rsplitn(2, |&b| b == b'/').next().unwrap_or(&branch[0..]);
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
