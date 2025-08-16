//! Per-branch metadata shared by stacks **and** loose branches.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Just a convenient alias – fewer raw `String`s wandering around.
pub type BranchId = String;

/// Simple life-cycle enum you can extend later.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BranchStatus {
    Draft,
    Open,
    Landed,
    Abandoned,
}

/// Everything we want to remember about a branch.
///
/// * `parent` is **always** set → no special cases.
/// * `pr_number` lets the CLI skip an API round-trip when it already knows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: BranchId,
    pub parent: BranchId,
    pub created: DateTime<Utc>,
    pub hosted: Option<DateTime<Utc>>,
    pub author: String,
    pub status: BranchStatus,
    pub depth: usize,
    pub pr_number: Option<u64>,
}

impl BranchInfo {
    /// Handy constructor used everywhere.
    pub fn new(name: BranchId, parent: BranchId, author: impl Into<String>, depth: usize) -> Self {
        Self {
            name,
            parent,
            created: Utc::now(),
            hosted: None,
            author: author.into(),
            status: BranchStatus::Draft,
            depth,
            pr_number: None,
        }
    }
}
