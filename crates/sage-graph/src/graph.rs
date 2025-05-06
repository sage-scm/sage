//! Core logic: stacks **+** loose branches.

use std::{
    collections::{HashMap, VecDeque},
    fmt,
    path::Path,
};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use sage_git::branch::get_default_branch;

use crate::{
    branch::{BranchId, BranchInfo, BranchStatus},
    persist,
};

/* ───────────────────────── Stack (tree under a name) ────────────────── */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    name:         String,
    root:         BranchId,
    branches:     HashMap<BranchId, BranchInfo>,
    children_map: HashMap<BranchId, Vec<BranchId>>,
}

impl Stack {
    /* ----- ctor ----- */

    pub fn new(name: impl Into<String>, root: BranchId, author: impl Into<String>) -> Self {
        let root_info = BranchInfo::new(root.clone(), root.clone(), author, 0);

        let mut branches     = HashMap::new();
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

    pub fn contains(&self, b: &str) -> bool {
        self.branches.contains_key(b)
    }
    pub fn info(&self, b: &str) -> Option<&BranchInfo> {
        self.branches.get(b)
    }

    /* ----- mutation ----- */

    pub fn add_child(
        &mut self,
        parent: &str,
        child: BranchId,
        author: Option<String>,
    ) -> Result<()> {
        if !self.contains(parent) {
            bail!("unknown branch \"{parent}\"");
        }
        if self.contains(&child) {
            bail!("branch \"{child}\" already exists in stack \"{}\"", self.name);
        }

        let depth = self.branches[parent].depth + 1;
        let info  = BranchInfo::new(child.clone(), parent.to_owned(), author.unwrap_or_default(), depth);

        self.branches.insert(child.clone(), info);
        self.children_map.entry(parent.to_owned()).or_default().push(child);
        Ok(())
    }
}

/* ───────────────────────────── SageGraph ────────────────────────────── */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageGraph {
    /* persistent ------------------------------------------ */
    stacks: HashMap<String, Stack>,
    loose:  HashMap<BranchId, BranchInfo>,

    /* runtime-only ---------------------------------------- */
    #[serde(skip)]
    branch_to_stack: HashMap<BranchId, String>,
    #[serde(skip)]
    loose_children:  HashMap<BranchId, Vec<BranchId>>,
}

impl Default for SageGraph {
    fn default() -> Self {
        Self {
            stacks: HashMap::new(),
            loose:  HashMap::new(),
            branch_to_stack: HashMap::new(),
            loose_children:  HashMap::new(),
        }
    }
}

/* ----- public I/O ----------------------------------------------------- */

impl SageGraph {
    pub fn load_or_default() -> Result<Self> {
        let mut g = persist::load()?;
        g.reindex();
        g.ensure_default_branch()?;
        Ok(g)
    }

    pub fn save(&self) -> Result<()> {
        persist::save(self)
    }
}

/* ----- runtime indexing ---------------------------------------------- */

impl SageGraph {
    fn reindex(&mut self) {
        self.branch_to_stack.clear();
        self.loose_children.clear();

        for (stack_name, stack) in &self.stacks {
            for id in stack.branches.keys() {
                self.branch_to_stack.insert(id.clone(), stack_name.clone());
            }
        }
        for (id, info) in &self.loose {
            self.loose_children
                .entry(info.parent.clone())
                .or_default()
                .push(id.clone());
        }
    }

    fn ensure_default_branch(&mut self) -> Result<()> {
        let default = get_default_branch().context("getting default branch")?;
        if self.tracks(&default) {
            return Ok(());
        }
        let info = BranchInfo::new(default.clone(), default.clone(), whoami::realname(), 0);
        self.loose.insert(default.clone(), info);
        self.reindex();
        Ok(())
    }
}

/* ----- stack API ------------------------------------------------------ */

impl SageGraph {
    pub fn new_stack(&mut self, name: impl Into<String>, root: BranchId) -> Result<()> {
        let name = name.into();
        if self.stacks.contains_key(&name) {
            bail!("stack \"{name}\" already exists");
        }
        if self.tracks(&root) {
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

    pub fn stack_of(&self, b: &str) -> Option<&Stack> {
        self.branch_to_stack.get(b).and_then(|s| self.stacks.get(s))
    }
}

/* ----- loose-branch API ---------------------------------------------- */

impl SageGraph {
    pub fn add_loose_branch(
        &mut self,
        branch: BranchId,
        parent: BranchId,
        author: impl Into<String>,
    ) -> Result<()> {
        if self.tracks(&branch) {
            bail!("branch \"{branch}\" already tracked");
        }
        if !self.tracks(&parent) {
            bail!("parent \"{parent}\" not tracked");
        }

        let depth = self.depth(&parent).unwrap_or(0) + 1;
        let info  = BranchInfo::new(branch.clone(), parent.clone(), author, depth);

        self.loose.insert(branch.clone(), info);
        self.loose_children.entry(parent).or_default().push(branch);
        Ok(())
    }

    pub fn is_loose(&self, b: &str) -> bool {
        self.loose.contains_key(b)
    }
}

/* ----- shared helpers ------------------------------------------------- */

impl SageGraph {
    pub fn tracks(&self, b: &str) -> bool {
        self.branch_to_stack.contains_key(b) || self.loose.contains_key(b)
    }

    pub fn info(&self, b: &str) -> Option<&BranchInfo> {
        self.stack_of(b).and_then(|s| s.info(b)).or_else(|| self.loose.get(b))
    }

    fn depth(&self, b: &str) -> Option<usize> {
        self.info(b).map(|i| i.depth)
    }
}
