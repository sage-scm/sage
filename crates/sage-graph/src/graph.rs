//! Core logic: stacks, loose branches, plus top-level `StackGraph`.

use anyhow::{Result, anyhow, bail};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt, path::Path};

use crate::{
    branch::{BranchId, BranchInfo},
    persist,
};

/* ─────────────────────────────────  Stack  ───────────────────────────── */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    name: String,
    root: BranchId,
    branches: HashMap<BranchId, BranchInfo>,
    children_map: HashMap<BranchId, Vec<BranchId>>,
}

impl Stack {
    /* ----- create ----- */

    pub fn new(name: impl Into<String>, root: BranchId, author: impl Into<String>) -> Self {
        // For a root branch, parent is itself
        let root_info = BranchInfo::new(root.clone(), root.clone(), author, 0);

        let mut branches = HashMap::new();
        let mut children_map = HashMap::new();

        branches.insert(root.clone(), root_info);
        children_map.insert(root.clone(), Vec::new());

        Self {
            name: name.into(),
            root,
            branches,
            children_map,
        }
    }

    /* ----- queries ----- */

    pub fn contains_branch(&self, b: &str) -> bool {
        self.branches.contains_key(b)
    }

    pub fn info(&self, b: &str) -> Option<&BranchInfo> {
        self.branches.get(b)
    }

    pub fn children_of(&self, b: &str) -> &[BranchId] {
        self.children_map
            .get(b)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn descendants(&self, b: &str) -> impl Iterator<Item = &BranchId> {
        let mut q: VecDeque<&BranchId> = VecDeque::new();
        if let Some((key, _)) = self.branches.get_key_value(b) {
            q.push_back(key);
        }
        std::iter::from_fn(move || {
            if let Some(next) = q.pop_front() {
                if let Some(children) = self.children_map.get(next) {
                    for c in children {
                        q.push_back(c);
                    }
                }
                Some(next)
            } else {
                None
            }
        })
    }

    /* ----- mutation ----- */

    pub fn add_child(
        &mut self,
        parent: &str,
        child: BranchId,
        author: Option<String>,
    ) -> Result<()> {
        if !self.contains_branch(parent) {
            bail!("unknown branch \"{parent}\"");
        }
        if self.contains_branch(&child) {
            bail!(
                "branch \"{child}\" already exists in stack \"{}\"",
                self.name
            );
        }

        let depth = self.branches[parent].depth + 1;
        let info = BranchInfo::new(
            child.clone(),
            parent.to_owned(),
            author.unwrap_or_else(|| "unknown".into()),
            depth,
        );

        self.branches.insert(child.clone(), info);
        self.children_map
            .entry(parent.to_owned())
            .or_default()
            .push(child);
        Ok(())
    }

    /* ----- pretty print ----- */

    fn fmt_branch(&self, f: &mut fmt::Formatter<'_>, b: &BranchId, indent: usize) -> fmt::Result {
        let info = &self.branches[b];
        writeln!(
            f,
            "{:indent$}• {} [{:?}]",
            "",
            b,
            info.status,
            indent = indent * 2
        )?;
        for c in self.children_of(b) {
            self.fmt_branch(f, c, indent + 1)?;
        }
        Ok(())
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "stack \"{}\":", self.name)?;
        self.fmt_branch(f, &self.root, 0)
    }
}

/* ─────────────────────────────  StackGraph  ─────────────────────────── */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackGraph {
    /* persistent -------------------------------------------------------- */
    stacks: HashMap<String, Stack>,
    loose: HashMap<BranchId, BranchInfo>,

    /* runtime-only ------------------------------------------------------ */
    #[serde(skip)]
    branch_to_stack: HashMap<BranchId, String>,
    #[serde(skip)]
    loose_children: HashMap<BranchId, Vec<BranchId>>,
}

impl Default for StackGraph {
    fn default() -> Self {
        Self {
            stacks: HashMap::new(),
            loose: HashMap::new(),
            branch_to_stack: HashMap::new(),
            loose_children: HashMap::new(),
        }
    }
}

/* ----- I/O ------------------------------------------------------------- */

impl StackGraph {
    /// Load the graph (or return default) from the repo’s `.git/sage_graph.json`.
    pub fn load_or_default() -> Result<Self> {
        let mut g = persist::load()?;
        g.reindex();
        Ok(g)
    }

    /// Persist the graph to the repo’s `.git/sage_graph.json`.
    pub fn save(&self) -> Result<()> {
        persist::save(self)
    }
}

/* ----- indexing helpers ----------------------------------------------- */

impl StackGraph {
    fn reindex(&mut self) {
        self.branch_to_stack.clear();
        self.loose_children.clear();

        for (stack_name, stack) in &self.stacks {
            for id in stack.branches.keys() {
                self.branch_to_stack.insert(id.clone(), stack_name.clone());
            }
        }
        for (id, info) in &self.loose {
            // Every loose branch has a required parent
            let p = &info.parent;
            self.loose_children
                .entry(p.clone())
                .or_default()
                .push(id.clone());
        }
    }
}

/* ----- stack API ------------------------------------------------------- */

impl StackGraph {
    pub fn new_stack(&mut self, name: impl Into<String>, root: BranchId) -> Result<()> {
        let name = name.into();
        if self.stacks.contains_key(&name) {
            bail!("stack \"{name}\" already exists");
        }
        if self.tracks_branch(&root) {
            bail!("branch \"{root}\" already tracked");
        }

        let stack = Stack::new(&name, root.clone(), whoami::realname());
        self.stacks.insert(name.clone(), stack);
        self.branch_to_stack.insert(root, name);
        Ok(())
    }

    pub fn add_stack_child(
        &mut self,
        stack_name: &str,
        parent: &str,
        child: BranchId,
        author: Option<String>,
    ) -> Result<()> {
        let stack = self
            .stacks
            .get_mut(stack_name)
            .ok_or_else(|| anyhow!("unknown stack \"{stack_name}\""))?;
        stack.add_child(parent, child.clone(), author)?;
        self.branch_to_stack.insert(child, stack_name.to_owned());
        Ok(())
    }

    pub fn stack_for_branch(&self, b: &str) -> Option<&Stack> {
        self.branch_to_stack
            .get(b)
            .and_then(|name| self.stacks.get(name))
    }
}

/* ----- loose-branch API ----------------------------------------------- */

impl StackGraph {
    /// Add a loose branch with a required parent.
    pub fn add_loose_branch(
        &mut self,
        branch: BranchId,
        parent: BranchId,
        author: impl Into<String>,
    ) -> Result<()> {
        if self.tracks_branch(&branch) {
            bail!("branch \"{branch}\" already tracked");
        }
        if !self.tracks_branch(&parent) {
            bail!("parent branch \"{parent}\" not tracked");
        }

        let depth = self.branch_depth(&parent).unwrap_or(0) + 1;
        let info = BranchInfo::new(branch.clone(), parent.clone(), author, depth);
        self.loose.insert(branch.clone(), info);

        self.loose_children.entry(parent).or_default().push(branch);
        Ok(())
    }

    pub fn is_loose(&self, b: &str) -> bool {
        self.loose.contains_key(b)
    }

    pub fn loose_children_of(&self, b: &str) -> &[BranchId] {
        self.loose_children
            .get(b)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

/* ----- shared helpers -------------------------------------------------- */

impl StackGraph {
    pub fn tracks_branch(&self, b: &str) -> bool {
        self.branch_to_stack.contains_key(b) || self.loose.contains_key(b)
    }

    pub fn branch_info(&self, b: &str) -> Option<&BranchInfo> {
        self.stack_for_branch(b)
            .and_then(|s| s.info(b))
            .or_else(|| self.loose.get(b))
    }

    fn branch_depth(&self, b: &str) -> Option<usize> {
        self.branch_info(b).map(|i| i.depth)
    }
}

/* ─────────────────────────────── Tests ──────────────────────────────── */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_and_loose() -> Result<()> {
        let mut g = StackGraph::default();
        g.new_stack("core", "core/base".into())?;

        g.add_stack_child("core", "core/base", "feat/a".into(), None)?;
        // `develop` must be tracked first (simulate adding develop as loose under itself)
        g.add_loose_branch("develop".into(), "develop".into(), "alice")?;
        g.add_loose_branch("hotfix/login-typo".into(), "develop".into(), "alice")?;

        assert!(g.tracks_branch("feat/a"));
        assert!(g.is_loose("hotfix/login-typo"));
        assert_eq!(g.branch_info("feat/a").unwrap().status, BranchStatus::Draft);
        Ok(())
    }
}
