//! Common metadata shared by *all* tracked branches.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Convenience alias = one less `String` to type.
pub type BranchId = String;

/// Basic life-cycle state.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BranchStatus {
    Draft,
    Open,
    Landed,
    Abandoned,
}

/// Everything we know about a branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: BranchId,
    /// Parent branch of this branch (always present).
    pub parent: BranchId,
    pub created: DateTime<Utc>,
    pub hosted: Option<DateTime<Utc>>,
    pub author: String,
    pub status: BranchStatus,
    pub depth: usize,
}

impl BranchInfo {
    /// Constructor used by both stack and loose branches.
    /// Constructor. `parent` must be provided (e.g. for a root branch, pass its own name).
    pub fn new(name: BranchId, parent: BranchId, author: impl Into<String>, depth: usize) -> Self {
        Self {
            name,
            parent,
            created: Utc::now(),
            hosted: None,
            author: author.into(),
            status: BranchStatus::Draft,
            depth,
        }
    }
}
