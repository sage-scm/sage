use std::convert::TryFrom;

use anyhow::{Context, Result};
use gix::status::Item as StatusItem;
use gix::{ObjectId, diff::index::Change as IndexChange, progress::Discard, status as gix_status};

use gix_status::plumbing::index_as_worktree_with_renames::Summary;

use super::Repo;

impl Repo {
    pub fn is_dirty(&self) -> Result<bool> {
        Ok(self.repo.is_dirty()?)
    }

    pub fn is_clean(&self) -> Result<bool> {
        Ok(!self.is_dirty()?)
    }

    pub fn unstaged_files(&self) -> Result<Vec<String>> {
        let platform = self
            .repo
            .status(Discard)?
            .untracked_files(gix_status::UntrackedFiles::Files);

        let iter = platform.into_index_worktree_iter(Vec::new())?;
        let mut files = Vec::new();

        for entry in iter {
            let item = entry?;
            if let Some(summary) = item.summary()
                && !matches!(summary, Summary::Added)
            {
                files.push(item.rela_path().to_string());
            }
        }

        files.sort_unstable();
        files.dedup();
        Ok(files)
    }

    pub fn untracked_files(&self) -> Result<Vec<String>> {
        let platform = self
            .repo
            .status(Discard)?
            .untracked_files(gix_status::UntrackedFiles::Files);

        let iter = platform.into_index_worktree_iter(Vec::new())?;
        let mut files = Vec::new();

        for entry in iter {
            let item = entry?;
            if let Some(Summary::Added) = item.summary() {
                files.push(item.rela_path().to_string());
            }
        }

        files.sort_unstable();
        files.dedup();
        Ok(files)
    }

    pub fn staged_changes(&self) -> Result<Vec<String>> {
        let iter = self
            .repo
            .status(Discard)?
            .index_worktree_submodules(None)
            .index_worktree_options_mut(|opts| {
                opts.dirwalk_options = None;
            })
            .into_iter(Vec::new())?;

        let mut changes = Vec::new();

        for item in iter {
            let item = item?;
            if let StatusItem::TreeIndex(change) = item {
                collect_staged_change(&mut changes, change);
            }
        }

        changes.sort();
        changes.dedup();
        Ok(changes)
    }

    pub fn above_below(&self, branch: &str) -> Result<(i32, i32)> {
        let target_ref = if !branch.starts_with("refs/") {
            self.as_ref(branch)
        } else {
            branch.to_string()
        };
        let head_id = self.repo.head_id()?.detach();
        let target_id = self
            .repo
            .rev_parse_single(target_ref.as_str())
            .with_context(|| format!("Failed to resolve branch '{branch}'"))?
            .detach();

        let ahead = self.unique_commit_count(head_id, target_id)?;
        let behind = self.unique_commit_count(target_id, head_id)?;

        let ahead = i32::try_from(ahead).context("ahead commit count exceeds i32 range")?;
        let behind = i32::try_from(behind).context("behind commit count exceeds i32 range")?;

        Ok((ahead, behind))
    }

    fn unique_commit_count(&self, start: ObjectId, hide: ObjectId) -> Result<usize> {
        let walk = self.repo.rev_walk([start]).with_hidden([hide]).all()?;

        let mut count = 0usize;
        for commit in walk {
            commit?;
            count += 1;
        }

        Ok(count)
    }
}

fn collect_staged_change(output: &mut Vec<String>, change: IndexChange) {
    match change {
        IndexChange::Addition { location, .. } | IndexChange::Modification { location, .. } => {
            output.push(location.to_string());
        }
        IndexChange::Deletion { location, .. } => {
            output.push(location.to_string());
        }
        IndexChange::Rewrite {
            source_location,
            location,
            copy,
            ..
        } => {
            if copy {
                output.push(format!("{} -> {} (copy)", source_location, location));
            } else {
                output.push(format!("{} -> {}", source_location, location));
            }
        }
    }
}
