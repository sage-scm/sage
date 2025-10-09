use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub parent: String,
    pub created: DateTime<Utc>,
    pub hosted: Option<DateTime<Utc>>,
    pub author: String,
    pub depth: usize,
    pub pr_number: Option<u64>,
}

impl BranchInfo {
    pub fn new(name: String, parent: String, author: String, depth: usize) -> Self {
        Self {
            name,
            parent,
            created: Utc::now(),
            hosted: None,
            author,
            depth,
            pr_number: None,
        }
    }
}
