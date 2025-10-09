use anyhow::{Result, bail};
use hashbrown::HashMap;

use crate::branch::BranchInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    pub name: String,
    pub root: String,
    pub branches: HashMap<String, BranchInfo>,
    pub children: HashMap<String, Vec<String>>,
}

impl Stack {
    pub fn new(name: String, root: String, parent: String, depth: usize, author: String) -> Self {
        let info = BranchInfo::new(root.clone(), parent, author, depth);
        let mut branches = HashMap::new();
        branches.insert(root.clone(), info);
        let mut children = HashMap::new();
        children.insert(root.clone(), Vec::new());
        Self {
            name,
            root,
            branches,
            children,
        }
    }

    pub fn contains(&self, branch: &str) -> bool {
        self.branches.contains_key(branch)
    }

    pub fn info(&self, branch: &str) -> Option<&BranchInfo> {
        self.branches.get(branch)
    }

    pub fn parent_id(&self, branch: &str) -> Option<&String> {
        self.info(branch).map(|info| &info.parent)
    }

    pub fn children(&self, branch: &str) -> &[String] {
        self.children.get(branch).map_or(&[], Vec::as_slice)
    }

    pub fn all_branches(&self) -> Vec<String> {
        self.branches.keys().cloned().collect()
    }

    pub fn add_child(&mut self, parent: &str, child: String, author: String) -> Result<()> {
        if !self.contains(parent) {
            bail!("parent branch \"{parent}\" not found");
        }
        if self.contains(&child) {
            bail!("child branch \"{child}\" already exists");
        }
        let depth = self.branches[parent].depth + 1;
        let info = BranchInfo::new(child.clone(), parent.to_owned(), author, depth);
        self.branches.insert(child.clone(), info.clone());
        self.children
            .entry(parent.to_owned())
            .or_default()
            .push(child);
        self.ensure_no_cycle(&info.name)?;
        Ok(())
    }

    pub fn descendants(&self, start: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut to_visit = vec![start.to_owned()];
        while let Some(current) = to_visit.pop() {
            result.push(current.clone());
            if let Some(kids) = self.children.get(&current) {
                for kid in kids.iter().rev() {
                    to_visit.push(kid.clone());
                }
            }
        }
        result
    }

    pub fn ancestors(&self, start: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = start;
        while let Some(parent) = self.parent_id(current) {
            if parent == current {
                break;
            }
            result.push(parent.clone());
            current = parent;
        }
        result
    }

    fn ensure_no_cycle(&self, start: &str) -> Result<()> {
        let mut visited = std::collections::HashSet::new();
        let mut current = start;
        while let Some(parent) = self.parent_id(current) {
            if !visited.insert(parent.clone()) {
                bail!("cycle detection involving \"{start}\"");
            }
            current = parent;
        }
        Ok(())
    }
}
