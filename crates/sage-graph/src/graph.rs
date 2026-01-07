use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{BranchInfo, Stack};
use sage_git::Repo;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SageGraph {
    stacks: HashMap<String, Stack>,
    #[serde(default, alias = "loose")]
    loose_branches: HashMap<String, BranchInfo>,
    #[serde(skip)]
    branch_to_stack: HashMap<String, String>,
    #[serde(skip)]
    loose_children: HashMap<String, Vec<String>>,
    #[serde(skip)]
    repo_root: Option<PathBuf>,
    #[serde(skip)]
    git_dir: Option<PathBuf>,
}

impl SageGraph {
    pub fn load(repo: &Repo) -> Result<Self> {
        let path = Self::storage_path(repo);
        let data = match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => SageGraph::default(),
            Err(e) => return Err(e).context("reading graph file"),
        };
        let mut graph = data;
        graph.capture_repo_environment(repo);
        graph.rebuild_indexes();
        graph.add_default_branch_if_missing(repo)?;
        Ok(graph)
    }

    pub fn save(&mut self, repo: &Repo) -> Result<()> {
        self.capture_repo_environment(repo);
        let path = Self::storage_path(repo);
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json).context("writing graph file")
    }

    fn storage_path(repo: &Repo) -> PathBuf {
        repo.git_dir().join("sage_graph.json")
    }

    fn rebuild_indexes(&mut self) {
        self.branch_to_stack.clear();
        self.loose_children.clear();

        for (name, stack) in &self.stacks {
            for branch in stack.branches.keys() {
                self.branch_to_stack.insert(branch.clone(), name.clone());
            }
        }

        for (branch, info) in &self.loose_branches {
            self.loose_children
                .entry(info.parent.clone())
                .or_default()
                .push(branch.clone());
        }
    }

    fn add_default_branch_if_missing(&mut self, repo: &Repo) -> Result<()> {
        self.capture_repo_environment(repo);
        let default = repo.get_default_branch()?;
        if self.is_tracked(&default) {
            return Ok(());
        }
        let author = Self::author_name(repo)?;
        let info = BranchInfo::new(default.clone(), default.clone(), author, 0);
        self.loose_branches.insert(default, info);
        self.rebuild_indexes();
        Ok(())
    }

    pub fn create_stack(
        &mut self,
        repo: &Repo,
        name: String,
        root: String,
        parent: String,
    ) -> Result<()> {
        self.capture_repo_environment(repo);
        if self.stacks.contains_key(&name) {
            bail!("stack \"{name}\" exists");
        }
        if self.is_tracked(&root) {
            bail!("branch \"{root}\" tracked");
        }
        if !self.is_tracked(&parent) {
            bail!("parent \"{parent}\" not tracked");
        }

        let depth = self.get_depth(&parent).unwrap_or(0) + 1;
        let author = Self::author_name(repo)?;
        let stack = Stack::new(name.clone(), root.clone(), parent, depth, author);
        self.stacks.insert(name.clone(), stack);
        self.branch_to_stack.insert(root, name);
        Ok(())
    }

    pub fn add_to_stack(
        &mut self,
        repo: &Repo,
        stack_name: &str,
        parent: &str,
        child: String,
    ) -> Result<()> {
        self.capture_repo_environment(repo);
        let stack = self
            .stacks
            .get_mut(stack_name)
            .ok_or_else(|| anyhow::anyhow!("stack \"{stack_name}\" not found"))?;
        let author = Self::author_name(repo)?;
        let _ = stack.add_child(parent, child.clone(), author);
        self.branch_to_stack.insert(child, stack_name.to_owned());
        Ok(())
    }

    pub fn stack_for_branch(&self, branch: &str) -> Option<&Stack> {
        self.branch_to_stack
            .get(branch)
            .and_then(|name| self.stacks.get(name))
    }

    pub fn stack_name_for_branch(&self, branch: &str) -> Option<&String> {
        self.branch_to_stack.get(branch)
    }

    pub fn add_loose_branch(&mut self, repo: &Repo, branch: String, parent: String) -> Result<()> {
        self.capture_repo_environment(repo);
        if self.is_tracked(&branch) {
            bail!("branch \"{branch}\" tracked");
        }

        if !self.is_tracked(&parent) {
            bail!("parent \"{parent}\" not tracked");
        }
        let depth = self.get_depth(&parent).unwrap_or(0) + 1;
        let author = Self::author_name(repo)?;
        let info = BranchInfo::new(branch.clone(), parent.clone(), author, depth);
        self.loose_branches.insert(branch.clone(), info.clone());
        self.loose_children.entry(parent).or_default().push(branch);
        self.ensure_loose_no_cycle(&info.name)?;
        Ok(())
    }

    pub fn has_children(&self, branch: &str) -> bool {
        if !self.is_tracked(branch) {
            return false;
        } // We dont track the branch, so we cant tell.

        let is_loose = self.loose_branches.contains_key(branch);

        if is_loose {
            if let Some(children) = self.loose_children.get(branch) {
                let has_real_children = children.iter().any(|child| child != branch);
                if has_real_children {
                    return true;
                }
            }
            return false;
        }

        #[allow(clippy::collapsible_if)]
        if let Some(stack) = self.stack_for_branch(branch) {
            if !stack.children.is_empty() {
                return true;
            }
        }

        false
    }

    pub fn remove_loose_branch(&mut self, branch: &str) -> Result<()> {
        if self.has_children(branch) {
            bail!("branch \"{branch}\" has loose children");
        }
        let info = self
            .loose_branches
            .remove(branch)
            .ok_or_else(|| anyhow::anyhow!("branch \"{branch}\" is not a loose branch"))?;
        if let Some(children) = self.loose_children.get_mut(&info.parent) {
            if let Some(pos) = children.iter().position(|child| child == branch) {
                children.remove(pos);
            }
            if children.is_empty() {
                self.loose_children.remove(&info.parent);
            }
        }
        self.loose_children.remove(branch);
        Ok(())
    }

    pub fn is_loose(&self, branch: &str) -> bool {
        self.loose_branches.contains_key(branch)
    }

    pub fn is_tracked(&self, branch: &str) -> bool {
        self.branch_to_stack.contains_key(branch) || self.loose_branches.contains_key(branch)
    }

    pub fn get_info(&self, branch: &str) -> Option<&BranchInfo> {
        self.stack_for_branch(branch)
            .and_then(|s| s.info(branch))
            .or_else(|| self.loose_branches.get(branch))
    }

    fn get_depth(&self, branch: &str) -> Option<usize> {
        self.get_info(branch).map(|info| info.depth)
    }

    pub fn in_same_stack(&self, first: &str, second: &str) -> bool {
        match (
            self.stack_name_for_branch(first),
            self.stack_name_for_branch(second),
        ) {
            (Some(one), Some(two)) => one == two,
            _ => false,
        }
    }

    fn ensure_loose_no_cycle(&self, start: &str) -> Result<()> {
        let mut visited = std::collections::HashSet::new();
        let mut current = start;
        while let Some(info) = self.loose_branches.get(current) {
            if info.parent == *current {
                if info.depth == 0 {
                    break;
                } else {
                    bail!("cycle in loose branches involving \"{start}\"");
                }
            }
            if !visited.insert(info.parent.clone()) {
                bail!("cycle in loose branches involving \"{start}\"");
            }
            current = &info.parent;
        }
        Ok(())
    }

    pub fn repo_root(&self) -> Option<&PathBuf> {
        self.repo_root.as_ref()
    }

    pub fn git_dir(&self) -> Option<&PathBuf> {
        self.git_dir.as_ref()
    }

    pub fn storage_path_cached(&self) -> Option<PathBuf> {
        self.git_dir.as_ref().map(|dir| dir.join("sage_graph.json"))
    }

    fn author_name(repo: &Repo) -> Result<String> {
        Ok(repo
            .author_name()?
            .unwrap_or_else(|| String::from("unknown")))
    }

    fn capture_repo_environment(&mut self, repo: &Repo) {
        self.repo_root = Some(repo.repo_root());
        self.git_dir = Some(repo.git_dir());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sage_git::{Repo, testing::TestRepo};

    #[test]
    fn same_stack_checks() {
        let repo = test_repo();
        let mut graph = graph_with_main(&repo);

        graph
            .create_stack(
                &repo,
                "feat".to_owned(),
                "feat/base".to_owned(),
                "main".to_owned(),
            )
            .unwrap();
        graph
            .add_to_stack(&repo, "feat", "feat/base", "feat/child".to_owned())
            .unwrap();

        graph
            .add_loose_branch(&repo, "hotfix".to_owned(), "feat/base".to_owned())
            .unwrap();

        let feat_base = graph.get_info("feat/base").unwrap();
        assert_eq!(feat_base.author, "Test User");
        let feat_child = graph.get_info("feat/child").unwrap();
        assert_eq!(feat_child.author, "Test User");
        let hotfix = graph.get_info("hotfix").unwrap();
        assert_eq!(hotfix.author, "Test User");

        assert!(graph.in_same_stack("feat/base", "feat/child"));
        assert!(!graph.in_same_stack("feat/base", "hotfix"));
        assert!(!graph.in_same_stack("feat/base", "unknown"));
    }

    #[test]
    fn stack_queries() {
        let repo = test_repo();
        let mut graph = graph_with_main(&repo);

        graph
            .create_stack(
                &repo,
                "feat".to_owned(),
                "feat/base".to_owned(),
                "main".to_owned(),
            )
            .unwrap();
        graph
            .add_to_stack(&repo, "feat", "feat/base", "feat/one".to_owned())
            .unwrap();
        graph
            .add_to_stack(&repo, "feat", "feat/base", "feat/two".to_owned())
            .unwrap();

        let stack = graph.stack_for_branch("feat/base").unwrap();
        let kids = stack.children("feat/base");
        assert_eq!(kids.len(), 2);
        assert!(kids.contains(&"feat/one".to_owned()));
        assert!(kids.contains(&"feat/two".to_owned()));

        let all = stack.all_branches();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn repo_context_captured() {
        let repo = test_repo();
        let mut graph = graph_with_main(&repo);

        graph
            .create_stack(
                &repo,
                "feat".to_owned(),
                "feat/base".to_owned(),
                "main".to_owned(),
            )
            .unwrap();

        let repo_root = graph.repo_root().expect("repo root cached");
        assert_eq!(repo_root.as_path(), repo.path());

        let cached_path = graph.storage_path_cached().expect("storage path cached");
        assert_eq!(cached_path, repo.git_dir().join("sage_graph.json"));
    }

    #[test]
    fn add_loose_branch_requires_tracked_parent() {
        let repo = test_repo();
        let mut graph = graph_with_main(&repo);
        let err = graph
            .add_loose_branch(&repo, "feature".to_owned(), "unknown".to_owned())
            .expect_err("untracked parent rejected");
        assert!(
            format!("{err:?}").contains("not tracked"),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn create_stack_rejects_duplicates() {
        let repo = test_repo();
        let mut graph = graph_with_main(&repo);
        graph
            .create_stack(
                &repo,
                "feat".to_owned(),
                "feat/base".to_owned(),
                "main".to_owned(),
            )
            .expect("first stack succeeds");
        let err = graph
            .create_stack(
                &repo,
                "feat".to_owned(),
                "feat/other".to_owned(),
                "main".to_owned(),
            )
            .expect_err("duplicate stack rejected");
        assert!(
            format!("{err:?}").contains("exists"),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn test_loose_no_cycle() {
        let repo = test_repo();
        let mut graph = graph_with_main(&repo);
        graph
            .add_loose_branch(&repo, "hotfix".to_owned(), "main".to_owned())
            .unwrap();

        assert!(graph.ensure_loose_no_cycle("hotfix").is_ok());
        assert!(graph.is_loose("hotfix"));
        assert!(!graph.is_loose("main"));
    }

    #[test]
    fn remove_loose_branch() {
        let repo = test_repo();
        let mut graph = graph_with_main(&repo);
        graph
            .add_loose_branch(&repo, "hotfix".to_owned(), "main".to_owned())
            .unwrap();

        assert!(graph.is_loose("hotfix"));
        assert!(graph.is_loose("main"));

        graph.remove_loose_branch("hotfix").unwrap();

        assert!(!graph.is_loose("hotfix"));
        assert!(graph.is_loose("main"));
    }

    fn test_repo() -> TestRepo {
        TestRepo::builder()
            .with_initial_commit()
            .build()
            .expect("temp repo")
    }

    fn graph_with_main(repo: &Repo) -> SageGraph {
        let author = repo
            .author_name()
            .expect("author query")
            .expect("author configured");
        let mut graph = SageGraph::default();
        graph.loose_branches.insert(
            "main".to_owned(),
            BranchInfo::new("main".to_owned(), "main".to_owned(), author, 0),
        );
        graph.rebuild_indexes();
        graph
    }
}
