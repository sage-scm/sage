use anyhow::Result;
use chrono::{DateTime, Utc};
use gix::revision::walk::Sorting;
use gix::{Id, traverse::commit::simple::CommitTimeOrder};

use super::Repo;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    pub date: String,
    pub author: String,
}

impl Repo {
    pub fn create_commit(&self, message: &str, allow_empty: bool, amend: bool) -> Result<()> {
        let mut command = self.git()?.arg("commit");
        if allow_empty {
            command = command.arg("--allow-empty");
        }
        if amend {
            command = command.arg("--amend");
        }
        command.arg("-m").arg(message).run()
    }

    pub fn get_current_commit(&self) -> Result<Id<'_>> {
        let found = self.repo.head_commit()?;
        Ok(found.id())
    }

    pub fn get_commits(&self, limit: Option<usize>) -> Result<Vec<Commit>> {
        if matches!(limit, Some(0)) {
            return Ok(Vec::new());
        }

        let branch = self.get_current_branch()?;
        let mut reference = self
            .repo
            .find_reference(&format!("refs/heads/{}", branch))?;
        let head_commit = reference.peel_to_commit()?;

        let walk = self
            .repo
            .rev_walk([head_commit.id()])
            .use_commit_graph(false)
            .sorting(Sorting::ByCommitTime(CommitTimeOrder::NewestFirst))
            .all()?;

        let mut commits = Vec::new();
        for info_res in walk {
            let info = info_res?;
            let id = info.id;

            let commit = self.repo.find_commit(id)?;

            let hash = commit.id().to_hex().to_string()[0..8].to_string();
            let message = String::from_utf8_lossy(commit.message_raw()?).to_string();
            let date = if let Some(dt) = DateTime::<Utc>::from_timestamp(commit.time()?.seconds, 0)
            {
                format!("{}", dt.format("%a %b %d %Y"))
            } else {
                "Unknown date".to_string()
            };

            let author = commit.author()?.name.to_string();

            commits.push(Commit {
                hash,
                message,
                date,
                author,
            });

            if let Some(limit) = limit
                && commits.len() >= limit
            {
                break;
            }
        }

        Ok(commits)
    }
}
