//! **sage-graph** – keep a tiny JSON record of every branch in a repo,
//! stacked _or_ loose, with zero mental overhead.
//!
//! ```no_run
//! use sage_graph::{BranchStatus, SageGraph};
//!
//! let mut g = SageGraph::load_or_default()?;
//!
//! /* stack */
//! g.new_stack("payments", "payments/base")?;
//! g.add_stack_child("payments", "payments/base", "feat/payment-limits", None)?;
//!
//! /* loose */
//! g.add_loose_branch("hotfix/login-typo", "develop", "Brayden")?;
//!
//! g.save()?;
//! # anyhow::Ok(())
//! ```

pub mod branch;
pub mod persist;
pub mod graph;

pub use branch::{BranchId, BranchInfo, BranchStatus};
pub use graph::{SageGraph, Stack};