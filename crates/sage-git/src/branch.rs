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
        FullName, Target,
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

    pub fn get_default_branch(&self) -> Result<String> {
        let reference = self
            .repo
            .find_reference("refs/remotes/origin/HEAD")
            .context("refs/remotes/origin/HEAD not found")?;

        let branch = reference.name().shorten().to_string();

        if branch.is_empty() {
            bail!("Unable to determine default branch");
        }

        Ok(branch)
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
