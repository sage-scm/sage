//! `stack_graph` – minimal branch & stack tracking for Git repos.
//!
//! *Everything* is in two files (`branch.rs`, `graph.rs`) plus tiny I/O glue.
//!
//! All public functions return `anyhow::Result<T>` so you can use the `?`
//! operator everywhere without thinking about custom error types.
//!
//! ```no_run
//! use stack_graph::{BranchStatus, StackGraph};
//!
//! let mut g = StackGraph::load_or_default(".")?;
//!
//! // ---- stack example ----
//! g.new_stack("payments", "payments/base")?;
//! g.add_stack_child("payments", "payments/base", "feat/credit-limits", None)?;
//!
//! // ---- loose branch example ----
//! g.add_loose_branch("hotfix/login-typo", Some("develop"), "Brayden")?;
//!
//! g.save(".")?;
//! # anyhow::Ok(())
//! ```

pub mod branch;
pub mod graph;
pub mod persist;

pub use branch::{BranchId, BranchInfo, BranchStatus};
pub use graph::{Stack, StackGraph};
