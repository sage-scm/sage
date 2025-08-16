use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::process::Command;

use crate::branch::get_current;

/// Represents the type of status for a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusType {
    /// File is staged (added to the index)
    Staged,
    /// File is modified but not staged
    Unstaged,
    /// File is untracked
    Untracked,
}

/// Represents a single file status entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEntry {
    /// Path of the file relative to the repository root
    pub path: String,
    /// Type of status (staged, unstaged, untracked)
    pub status_type: StatusType,
    /// Git status code (e.g., "M", "A", "??", etc.)
    pub status_code: String,
}

/// Represents the overall status of the repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    /// List of all status entries
    pub entries: Vec<StatusEntry>,
    /// Number of commits ahead of the remote branch
    pub ahead: usize,
    /// Number of commits behind the remote branch
    pub behind: usize,
}

/// Display options for formatting git status output
#[derive(Debug, Clone)]
pub struct DisplayOptions {
    pub show_branch_info: bool,
    pub show_staged: bool,
    pub show_unstaged: bool,
    pub show_untracked: bool,
    pub show_ignored: bool,
    pub use_symbols: bool,
    pub group_by_status: bool,
    pub max_path_length: Option<usize>,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            show_branch_info: true,
            show_staged: true,
            show_unstaged: true,
            show_untracked: true,
            show_ignored: false,
            use_symbols: true,
            group_by_status: true,
            max_path_length: None,
        }
    }
}

/// Git file status with symbols for display
pub struct StatusSymbols {
    pub added: &'static str,
    pub modified: &'static str,
    pub deleted: &'static str,
    pub renamed: &'static str,
    pub copied: &'static str,
    pub untracked: &'static str,
    pub ignored: &'static str,
}

impl Default for StatusSymbols {
    fn default() -> Self {
        Self {
            added: "A",
            modified: "M",
            deleted: "D",
            renamed: "R",
            copied: "C",
            untracked: "?",
            ignored: "!",
        }
    }
}

impl Display for GitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use default display options
        self.fmt_with_options(f, &DisplayOptions::default(), &StatusSymbols::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub current_branch: String,
    pub upstream_branch: Option<String>,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub has_stash: bool,
    pub staged_added: Vec<String>,
    pub staged_modified: Vec<String>,
    pub staged_deleted: Vec<String>,
    pub staged_renamed: Vec<(String, String)>,
    pub staged_copied: Vec<(String, String)>,
    pub unstaged_modified: Vec<String>,
    pub unstaged_deleted: Vec<String>,
    pub unstaged_added: Vec<String>,
    pub untracked: Vec<String>,
    pub ignored: Vec<String>,
    pub staged_modified_unstaged_modified: Vec<String>,
    pub staged_added_unstaged_modified: Vec<String>,
    pub staged_added_unstaged_deleted: Vec<String>,
    pub staged_deleted_unstaged_modified: Vec<String>,
    pub staged_renamed_unstaged_modified: Vec<String>,
    pub staged_copied_unstaged_modified: Vec<String>,
}

impl GitStatus {
    /// Format status with custom options
    pub fn fmt_with_options(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        options: &DisplayOptions,
        symbols: &StatusSymbols,
    ) -> std::fmt::Result {
        let mut lines = Vec::with_capacity(50); // Pre-allocate reasonable capacity

        // Branch information
        if options.show_branch_info {
            lines.push(format!("On branch {}", self.current_branch));

            if let Some(upstream) = &self.upstream_branch {
                let relation = if self.ahead_count > 0 && self.behind_count > 0 {
                    format!("ahead {}, behind {}", self.ahead_count, self.behind_count)
                } else if self.ahead_count > 0 {
                    format!("ahead {}", self.ahead_count)
                } else if self.behind_count > 0 {
                    format!("behind {}", self.behind_count)
                } else {
                    "up to date".to_string()
                };

                lines.push(format!("Your branch is {relation} with '{upstream}'"));
            } else if !self.current_branch.is_empty() {
                lines.push("Your branch is not tracking a remote branch".to_string());
            }

            if self.has_stash {
                lines.push("You have stashed changes".to_string());
            }

            lines.push(String::new()); // Empty line after branch info
        }

        let has_staged = !self.staged_added.is_empty()
            || !self.staged_modified.is_empty()
            || !self.staged_deleted.is_empty()
            || !self.staged_renamed.is_empty()
            || !self.staged_copied.is_empty()
            || !self.staged_modified_unstaged_modified.is_empty()
            || !self.staged_added_unstaged_modified.is_empty()
            || !self.staged_added_unstaged_deleted.is_empty()
            || !self.staged_deleted_unstaged_modified.is_empty()
            || !self.staged_renamed_unstaged_modified.is_empty()
            || !self.staged_copied_unstaged_modified.is_empty();

        let has_unstaged = !self.unstaged_modified.is_empty()
            || !self.unstaged_deleted.is_empty()
            || !self.unstaged_added.is_empty()
            || !self.staged_modified_unstaged_modified.is_empty()
            || !self.staged_added_unstaged_modified.is_empty()
            || !self.staged_added_unstaged_deleted.is_empty()
            || !self.staged_deleted_unstaged_modified.is_empty()
            || !self.staged_renamed_unstaged_modified.is_empty()
            || !self.staged_copied_unstaged_modified.is_empty();

        // Show summary if nothing to display
        if !has_staged && !has_unstaged && self.untracked.is_empty() && self.ignored.is_empty() {
            lines.push("Nothing to commit, working tree clean".to_string());
        }

        // Staged changes
        if options.show_staged && has_staged {
            lines.push("Changes to be committed:".to_string());

            if options.group_by_status {
                // Add staged added files
                for item in &self.staged_added {
                    let path = self.maybe_truncate_path(item, options.max_path_length);
                    lines.push(format!("  {:<2} {}", symbols.added, path));
                }

                // Add staged modified files
                for item in &self.staged_modified {
                    let path = self.maybe_truncate_path(item, options.max_path_length);
                    lines.push(format!("  {:<2} {}", symbols.modified, path));
                }

                // Add staged deleted files
                for item in &self.staged_deleted {
                    let path = self.maybe_truncate_path(item, options.max_path_length);
                    lines.push(format!("  {:<2} {}", symbols.deleted, path));
                }

                // Add renamed files
                for (from, to) in &self.staged_renamed {
                    let from_path = self.maybe_truncate_path(from, options.max_path_length);
                    let to_path = self.maybe_truncate_path(to, options.max_path_length);
                    lines.push(format!(
                        "  {:<2} {} -> {}",
                        symbols.renamed, from_path, to_path
                    ));
                }

                // Add copied files
                for (from, to) in &self.staged_copied {
                    let from_path = self.maybe_truncate_path(from, options.max_path_length);
                    let to_path = self.maybe_truncate_path(to, options.max_path_length);
                    lines.push(format!(
                        "  {:<2} {} -> {}",
                        symbols.copied, from_path, to_path
                    ));
                }
            }

            // Combined states
            // Add staged and unstaged modified files
            for item in &self.staged_modified_unstaged_modified {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!(
                    "  {}{}  {}",
                    symbols.modified, symbols.modified, path
                ));
            }

            // Add staged added and unstaged modified files
            for item in &self.staged_added_unstaged_modified {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!("  {}{}  {}", symbols.added, symbols.modified, path));
            }

            // Add staged added and unstaged deleted files
            for item in &self.staged_added_unstaged_deleted {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!("  {}{}  {}", symbols.added, symbols.deleted, path));
            }

            // Add staged deleted and unstaged modified files
            for item in &self.staged_deleted_unstaged_modified {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!(
                    "  {}{}  {}",
                    symbols.deleted, symbols.modified, path
                ));
            }

            // Add staged renamed and unstaged modified files
            for item in &self.staged_renamed_unstaged_modified {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!(
                    "  {}{}  {}",
                    symbols.renamed, symbols.modified, path
                ));
            }

            // Add staged copied and unstaged modified files
            for item in &self.staged_copied_unstaged_modified {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!(
                    "  {}{}  {}",
                    symbols.copied, symbols.modified, path
                ));
            }

            lines.push(String::new()); // Empty line after section
        }

        // Unstaged changes
        if options.show_unstaged && has_unstaged {
            lines.push("Changes not staged for commit:".to_string());

            // Add unstaged modified files
            for item in &self.unstaged_modified {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!("  {:<2} {}", symbols.modified, path));
            }

            // Add unstaged deleted files
            for item in &self.unstaged_deleted {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!("  {:<2} {}", symbols.deleted, path));
            }

            // Add unstaged added files
            for item in &self.unstaged_added {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!("  {:<2} {}", symbols.added, path));
            }

            lines.push(String::new()); // Empty line after section
        }

        // Untracked files
        if options.show_untracked && !self.untracked.is_empty() {
            lines.push("Untracked files:".to_string());
            for item in &self.untracked {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!("  {:<2} {}", symbols.untracked, path));
            }
            lines.push(String::new()); // Empty line after section
        }

        // Ignored files
        if options.show_ignored && !self.ignored.is_empty() {
            lines.push("Ignored files:".to_string());
            for item in &self.ignored {
                let path = self.maybe_truncate_path(item, options.max_path_length);
                lines.push(format!("  {:<2} {}", symbols.ignored, path));
            }
        }

        write!(f, "{}", lines.join("\n"))
    }

    /// Create a simple summary of the status
    #[inline]
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        let staged_count = self.staged_files_count();
        let unstaged_count = self.unstaged_files_count();
        let untracked_count = self.untracked.len();

        if staged_count > 0 {
            parts.push(format!("{staged_count} staged"));
        }

        if unstaged_count > 0 {
            parts.push(format!("{unstaged_count} not staged"));
        }

        if untracked_count > 0 {
            parts.push(format!("{untracked_count} untracked"));
        }

        if parts.is_empty() {
            "clean".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Returns a compact status string (e.g., for prompts)
    pub fn compact_status(&self) -> String {
        let mut status = String::with_capacity(50); // Pre-allocate reasonable capacity

        if !self.current_branch.is_empty() {
            status.push_str(&self.current_branch);
        } else {
            status.push_str("detached");
        }

        let staged = self.staged_files_count();
        let unstaged = self.unstaged_files_count();
        let untracked = self.untracked.len();

        if staged > 0 || unstaged > 0 || untracked > 0 {
            status.push_str(" [");

            if staged > 0 {
                status.push_str(&format!("+{staged}"));
            }

            if unstaged > 0 {
                status.push_str(&format!("!{unstaged}"));
            }

            if untracked > 0 {
                status.push_str(&format!("?{untracked}"));
            }

            status.push(']');
        }

        if self.ahead_count > 0 {
            status.push_str(&format!(" ↑{}", self.ahead_count));
        }

        if self.behind_count > 0 {
            status.push_str(&format!(" ↓{}", self.behind_count));
        }

        if self.has_stash {
            status.push_str(" $");
        }

        status
    }

    // Helper utility methods

    /// Truncate path if max_length is specified
    #[inline]
    fn maybe_truncate_path(&self, path: &str, max_length: Option<usize>) -> String {
        if let Some(max) = max_length {
            if path.len() > max {
                let mut truncated = String::with_capacity(max + 3);
                truncated.push_str("...");
                truncated.push_str(&path[path.len().saturating_sub(max - 3)..]);
                return truncated;
            }
        }
        path.to_string()
    }

    /// Checks if there are any changes (staged or unstaged)
    #[inline]
    pub fn has_changes(&self) -> bool {
        !self.staged_added.is_empty()
            || !self.staged_modified.is_empty()
            || !self.staged_deleted.is_empty()
            || !self.staged_renamed.is_empty()
            || !self.staged_copied.is_empty()
            || !self.unstaged_modified.is_empty()
            || !self.unstaged_deleted.is_empty()
            || !self.unstaged_added.is_empty()
            || !self.untracked.is_empty()
            || !self.staged_modified_unstaged_modified.is_empty()
            || !self.staged_added_unstaged_modified.is_empty()
            || !self.staged_added_unstaged_deleted.is_empty()
            || !self.staged_deleted_unstaged_modified.is_empty()
            || !self.staged_renamed_unstaged_modified.is_empty()
            || !self.staged_copied_unstaged_modified.is_empty()
    }

    /// Checks if there are any staged changes
    #[inline]
    pub fn has_staged_changes(&self) -> bool {
        !self.staged_added.is_empty()
            || !self.staged_modified.is_empty()
            || !self.staged_deleted.is_empty()
            || !self.staged_renamed.is_empty()
            || !self.staged_copied.is_empty()
            || !self.staged_modified_unstaged_modified.is_empty()
            || !self.staged_added_unstaged_modified.is_empty()
            || !self.staged_added_unstaged_deleted.is_empty()
            || !self.staged_deleted_unstaged_modified.is_empty()
            || !self.staged_renamed_unstaged_modified.is_empty()
            || !self.staged_copied_unstaged_modified.is_empty()
    }

    /// Checks if there are any unstaged changes
    #[inline]
    pub fn has_unstaged_changes(&self) -> bool {
        !self.unstaged_modified.is_empty()
            || !self.unstaged_deleted.is_empty()
            || !self.unstaged_added.is_empty()
            || !self.staged_modified_unstaged_modified.is_empty()
            || !self.staged_added_unstaged_modified.is_empty()
            || !self.staged_added_unstaged_deleted.is_empty()
            || !self.staged_deleted_unstaged_modified.is_empty()
            || !self.staged_renamed_unstaged_modified.is_empty()
            || !self.staged_copied_unstaged_modified.is_empty()
    }

    /// Checks if there are any untracked files
    #[inline]
    pub fn has_untracked(&self) -> bool {
        !self.untracked.is_empty()
    }

    /// Count total number of staged files
    #[inline]
    pub fn staged_files_count(&self) -> usize {
        self.staged_added.len()
            + self.staged_modified.len()
            + self.staged_deleted.len()
            + self.staged_renamed.len()
            + self.staged_copied.len()
    }

    /// Count total number of unstaged files
    #[inline]
    pub fn unstaged_files_count(&self) -> usize {
        self.unstaged_modified.len() + self.unstaged_deleted.len() + self.unstaged_added.len()
    }

    /// Count total number of combined status files
    #[inline]
    pub fn combined_status_files_count(&self) -> usize {
        self.staged_modified_unstaged_modified.len()
            + self.staged_added_unstaged_modified.len()
            + self.staged_added_unstaged_deleted.len()
            + self.staged_deleted_unstaged_modified.len()
            + self.staged_renamed_unstaged_modified.len()
            + self.staged_copied_unstaged_modified.len()
    }

    /// Get all modified files (both staged and unstaged)
    pub fn all_modified_files(&self) -> Vec<String> {
        let total_size = self.staged_modified.len()
            + self.unstaged_modified.len()
            + self.staged_modified_unstaged_modified.len()
            + self.staged_added_unstaged_modified.len()
            + self.staged_deleted_unstaged_modified.len()
            + self.staged_renamed_unstaged_modified.len()
            + self.staged_copied_unstaged_modified.len();

        let mut files = Vec::with_capacity(total_size);
        files.extend_from_slice(&self.staged_modified);
        files.extend_from_slice(&self.unstaged_modified);
        files.extend_from_slice(&self.staged_modified_unstaged_modified);
        files.extend_from_slice(&self.staged_added_unstaged_modified);
        files.extend_from_slice(&self.staged_deleted_unstaged_modified);
        files.extend_from_slice(&self.staged_renamed_unstaged_modified);
        files.extend_from_slice(&self.staged_copied_unstaged_modified);
        files
    }

    /// Get all added files (both staged and unstaged)
    pub fn all_added_files(&self) -> Vec<String> {
        let total_size = self.staged_added.len() + self.unstaged_added.len();
        let mut files = Vec::with_capacity(total_size);
        files.extend_from_slice(&self.staged_added);
        files.extend_from_slice(&self.unstaged_added);
        files
    }

    /// Get all deleted files (both staged and unstaged)
    pub fn all_deleted_files(&self) -> Vec<String> {
        let total_size = self.staged_deleted.len()
            + self.unstaged_deleted.len()
            + self.staged_added_unstaged_deleted.len();
        let mut files = Vec::with_capacity(total_size);
        files.extend_from_slice(&self.staged_deleted);
        files.extend_from_slice(&self.unstaged_deleted);
        files.extend_from_slice(&self.staged_added_unstaged_deleted);
        files
    }

    /// Get all renamed files
    pub fn all_renamed_files(&self) -> Vec<(String, String)> {
        self.staged_renamed.clone()
    }

    /// Get all copied files
    pub fn all_copied_files(&self) -> Vec<(String, String)> {
        self.staged_copied.clone()
    }

    /// Check if a specific file is staged
    pub fn is_file_staged(&self, path: &str) -> bool {
        self.staged_added.contains(&path.to_string())
            || self.staged_modified.contains(&path.to_string())
            || self.staged_deleted.contains(&path.to_string())
            || self.staged_renamed.iter().any(|(_, to)| to == path)
            || self.staged_copied.iter().any(|(_, to)| to == path)
            || self
                .staged_modified_unstaged_modified
                .contains(&path.to_string())
            || self
                .staged_added_unstaged_modified
                .contains(&path.to_string())
            || self
                .staged_added_unstaged_deleted
                .contains(&path.to_string())
            || self
                .staged_deleted_unstaged_modified
                .contains(&path.to_string())
            || self
                .staged_renamed_unstaged_modified
                .contains(&path.to_string())
            || self
                .staged_copied_unstaged_modified
                .contains(&path.to_string())
    }

    /// Check if a specific file is unstaged
    pub fn is_file_unstaged(&self, path: &str) -> bool {
        self.unstaged_modified.contains(&path.to_string())
            || self.unstaged_deleted.contains(&path.to_string())
            || self.unstaged_added.contains(&path.to_string())
            || self
                .staged_modified_unstaged_modified
                .contains(&path.to_string())
            || self
                .staged_added_unstaged_modified
                .contains(&path.to_string())
            || self
                .staged_added_unstaged_deleted
                .contains(&path.to_string())
            || self
                .staged_deleted_unstaged_modified
                .contains(&path.to_string())
            || self
                .staged_renamed_unstaged_modified
                .contains(&path.to_string())
            || self
                .staged_copied_unstaged_modified
                .contains(&path.to_string())
    }

    /// Check if a specific file is untracked
    pub fn is_file_untracked(&self, path: &str) -> bool {
        self.untracked.contains(&path.to_string())
    }

    /// Get the status of a specific file
    pub fn get_file_status(&self, path: &str) -> Vec<&'static str> {
        let path_str = path.to_string();
        let mut statuses = Vec::new();

        if self.staged_added.contains(&path_str) {
            statuses.push("staged added");
        }

        if self.staged_modified.contains(&path_str) {
            statuses.push("staged modified");
        }

        if self.staged_deleted.contains(&path_str) {
            statuses.push("staged deleted");
        }

        if self.staged_renamed.iter().any(|(_, to)| to == path) {
            statuses.push("staged renamed");
        }

        if self.staged_copied.iter().any(|(_, to)| to == path) {
            statuses.push("staged copied");
        }

        if self.unstaged_modified.contains(&path_str) {
            statuses.push("unstaged modified");
        }

        if self.unstaged_deleted.contains(&path_str) {
            statuses.push("unstaged deleted");
        }

        if self.unstaged_added.contains(&path_str) {
            statuses.push("unstaged added");
        }

        if self.staged_modified_unstaged_modified.contains(&path_str) {
            statuses.push("staged modified, unstaged modified");
        }

        if self.staged_added_unstaged_modified.contains(&path_str) {
            statuses.push("staged added, unstaged modified");
        }

        if self.staged_added_unstaged_deleted.contains(&path_str) {
            statuses.push("staged added, unstaged deleted");
        }

        if self.staged_deleted_unstaged_modified.contains(&path_str) {
            statuses.push("staged deleted, unstaged modified");
        }

        if self.staged_renamed_unstaged_modified.contains(&path_str) {
            statuses.push("staged renamed, unstaged modified");
        }

        if self.staged_copied_unstaged_modified.contains(&path_str) {
            statuses.push("staged copied, unstaged modified");
        }

        if self.untracked.contains(&path_str) {
            statuses.push("untracked");
        }

        if self.ignored.contains(&path_str) {
            statuses.push("ignored");
        }

        statuses
    }

    /// Filter the status to only include files in a given directory
    pub fn filter_by_directory(&self, directory: &str) -> GitStatus {
        let dir_path = if directory.ends_with('/') {
            directory.to_string()
        } else {
            format!("{directory}/")
        };

        let filter_vec = |files: &[String]| -> Vec<String> {
            files
                .iter()
                .filter(|file| file.starts_with(&dir_path) || file == &directory)
                .cloned()
                .collect()
        };

        let filter_pair_vec = |pairs: &[(String, String)]| -> Vec<(String, String)> {
            pairs
                .iter()
                .filter(|(from, to)| {
                    from.starts_with(&dir_path)
                        || from == directory
                        || to.starts_with(&dir_path)
                        || to == directory
                })
                .cloned()
                .collect()
        };

        GitStatus {
            current_branch: self.current_branch.clone(),
            upstream_branch: self.upstream_branch.clone(),
            ahead_count: self.ahead_count,
            behind_count: self.behind_count,
            has_stash: self.has_stash,

            staged_added: filter_vec(&self.staged_added),
            staged_modified: filter_vec(&self.staged_modified),
            staged_deleted: filter_vec(&self.staged_deleted),
            staged_renamed: filter_pair_vec(&self.staged_renamed),
            staged_copied: filter_pair_vec(&self.staged_copied),

            unstaged_modified: filter_vec(&self.unstaged_modified),
            unstaged_deleted: filter_vec(&self.unstaged_deleted),
            unstaged_added: filter_vec(&self.unstaged_added),

            untracked: filter_vec(&self.untracked),
            ignored: filter_vec(&self.ignored),

            staged_modified_unstaged_modified: filter_vec(&self.staged_modified_unstaged_modified),
            staged_added_unstaged_modified: filter_vec(&self.staged_added_unstaged_modified),
            staged_added_unstaged_deleted: filter_vec(&self.staged_added_unstaged_deleted),
            staged_deleted_unstaged_modified: filter_vec(&self.staged_deleted_unstaged_modified),
            staged_renamed_unstaged_modified: filter_vec(&self.staged_renamed_unstaged_modified),
            staged_copied_unstaged_modified: filter_vec(&self.staged_copied_unstaged_modified),
        }
    }

    /// Checks if the repository is clean (has no changes)
    #[inline]
    pub fn is_clean(&self) -> bool {
        !self.has_changes() && self.untracked.is_empty()
    }

    /// Checks if the repository is dirty (has changes)
    #[inline]
    pub fn is_dirty(&self) -> bool {
        !self.is_clean()
    }

    /// Checks if the local branch has diverged from its upstream branch
    /// A branch is considered diverged when it has both ahead and behind commits
    #[inline]
    pub fn is_diverged(&self) -> bool {
        self.ahead_count > 0 && self.behind_count > 0
    }

    /// Returns just the upstream status (ahead/behind) in a concise format
    pub fn upstream_status(&self) -> String {
        if self.ahead_count == 0 && self.behind_count == 0 {
            return String::new();
        }

        let mut status = String::with_capacity(15);
        status.push('[');

        if self.ahead_count > 0 {
            status.push_str(&format!("↑{}", self.ahead_count));
        }

        if self.behind_count > 0 {
            status.push_str(&format!("↓{}", self.behind_count));
        }

        status.push(']');
        status
    }

    /// Check if we need to push changes to remote
    pub fn needs_push(&self) -> bool {
        self.ahead_count > 0
    }

    /// Check if we need to pull changes from remote
    pub fn needs_pull(&self) -> bool {
        self.behind_count > 0
    }
}

/// Get the status of a specific branch without switching to it
pub fn branch_status(branch: &str) -> Result<GitStatus> {
    // Get the branch status without switching
    let (ahead, behind) = get_branch_ahead_behind(branch)?;
    let has_stash = has_stash()?;
    let upstream_branch = get_upstream_branch(branch)?;

    // For other branches, just get the basic status
    Ok(GitStatus {
        current_branch: branch.to_string(),
        upstream_branch,
        ahead_count: ahead,
        behind_count: behind,
        has_stash,
        ..Default::default()
    })
}

/// Get the ahead and behind counts for a branch relative to its upstream
fn get_branch_ahead_behind(branch: &str) -> Result<(usize, usize)> {
    // Get the upstream branch name
    let upstream = get_upstream_branch(branch)?;

    if let Some(upstream) = upstream {
        // Get the merge base between the branch and its upstream
        let merge_base = Command::new("git")
            .args(["merge-base", branch, &upstream])
            .output()?;

        if !merge_base.status.success() {
            return Ok((0, 0));
        }

        let merge_base = String::from_utf8(merge_base.stdout)?.trim().to_string();

        // Count commits ahead (in branch but not in upstream)
        let ahead = Command::new("git")
            .args([
                "rev-list",
                "--count",
                &format!("{merge_base}..{branch}"),
            ])
            .output()?;

        // Count commits behind (in upstream but not in branch)
        let behind = Command::new("git")
            .args([
                "rev-list",
                "--count",
                &format!("{}..{}", merge_base, &upstream),
            ])
            .output()?;

        let ahead_count = if ahead.status.success() {
            String::from_utf8(ahead.stdout)?.trim().parse().unwrap_or(0)
        } else {
            0
        };

        let behind_count = if behind.status.success() {
            String::from_utf8(behind.stdout)?
                .trim()
                .parse()
                .unwrap_or(0)
        } else {
            0
        };

        Ok((ahead_count, behind_count))
    } else {
        // No upstream set
        Ok((0, 0))
    }
}

/// Get the upstream branch name for a given branch
fn get_upstream_branch(branch: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .args([
            "rev-parse",
            "--abbrev-ref",
            &format!("{branch}@{{upstream}}"),
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let upstream = String::from_utf8(output.stdout)?.trim().to_string();
            Ok(Some(upstream))
        }
        _ => Ok(None), // No upstream set
    }
}

/// Check if there are any stashed changes
fn has_stash() -> Result<bool> {
    let output = Command::new("git").args(["stash", "list"]).output()?;

    Ok(!output.stdout.is_empty())
}

// Update the existing status function to use branch_status
pub fn status() -> Result<GitStatus> {
    let current_branch = get_current()?;
    let mut status = branch_status(&current_branch)?;

    // Parse the actual file status
    let result = Command::new("git")
        .args(["status", "--porcelain", "-z"])
        .output()?;

    if !result.status.success() {
        return Err(anyhow!(
            "Failed to get git status: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    let stdout = String::from_utf8(result.stdout)?;

    // Split by null terminator for proper handling of filenames with spaces
    for entry in stdout.split('\0') {
        if entry.is_empty() {
            continue;
        }

        // Git status format: XY filename
        // X = status in index, Y = status in working tree
        if entry.len() < 3 {
            continue;
        }

        let status_chars = &entry[0..2];
        let path = entry[3..].to_string();

        // Handle renamed/copied files which have format: XY oldname -> newname
        let (status_x, status_y) = (
            status_chars.chars().next().unwrap_or(' '),
            status_chars.chars().nth(1).unwrap_or(' '),
        );

        match (status_x, status_y) {
            // Untracked
            ('?', '?') => status.untracked.push(path),
            // Ignored
            ('!', '!') => status.ignored.push(path),
            // Added to index
            ('A', ' ') => status.staged_added.push(path),
            ('A', 'M') => {
                status.staged_added_unstaged_modified.push(path);
            }
            ('A', 'D') => {
                status.staged_added_unstaged_deleted.push(path);
            }
            // Modified
            ('M', ' ') => status.staged_modified.push(path),
            ('M', 'M') => {
                status.staged_modified_unstaged_modified.push(path);
            }
            ('M', 'D') => {
                status.staged_deleted_unstaged_modified.push(path);
            }
            (' ', 'M') => status.unstaged_modified.push(path),
            // Deleted
            ('D', ' ') => status.staged_deleted.push(path),
            ('D', 'M') => {
                status.staged_deleted_unstaged_modified.push(path);
            }
            (' ', 'D') => status.unstaged_deleted.push(path),
            // Renamed
            ('R', ' ') => {
                // For renamed files, parse the old -> new format
                if let Some(arrow_pos) = path.find(" -> ") {
                    let old_path = path[..arrow_pos].to_string();
                    let new_path = path[arrow_pos + 4..].to_string();
                    status.staged_renamed.push((old_path, new_path));
                }
            }
            ('R', 'M') => {
                // For renamed + modified files, we just track the new name
                if let Some(arrow_pos) = path.find(" -> ") {
                    let new_path = path[arrow_pos + 4..].to_string();
                    status.staged_renamed_unstaged_modified.push(new_path);
                }
            }
            // Copied
            ('C', ' ') => {
                // For copied files, parse the old -> new format
                if let Some(arrow_pos) = path.find(" -> ") {
                    let old_path = path[..arrow_pos].to_string();
                    let new_path = path[arrow_pos + 4..].to_string();
                    status.staged_copied.push((old_path, new_path));
                }
            }
            ('C', 'M') => {
                // For copied + modified files, we just track the new name
                if let Some(arrow_pos) = path.find(" -> ") {
                    let new_path = path[arrow_pos + 4..].to_string();
                    status.staged_copied_unstaged_modified.push(new_path);
                }
            }
            // Type changed
            ('T', ' ') => status.staged_modified.push(path),
            ('T', 'M') => {
                status.staged_modified_unstaged_modified.push(path);
            }
            (' ', 'T') => status.unstaged_modified.push(path),
            // Unmerged paths - treat as unstaged modified
            ('U', _) | (_, 'U') => status.unstaged_modified.push(path),
            // Unknown/other cases
            _ => {}
        }
    }

    Ok(status)
}

/// Get all status entries (staged, unstaged, untracked files)
pub fn get_status_entries() -> Result<Vec<StatusEntry>> {
    let result = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()?;

    if !result.status.success() {
        return Err(anyhow!(
            "Failed to get git status: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    let stdout = String::from_utf8(result.stdout)?;
    let mut entries = Vec::new();

    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }

        // Parse the status line
        // Format is XY PATH where X is the status in the index and Y is the status in the working tree
        let status_code = line.get(0..2).unwrap_or("  ").trim().to_string();
        let path = line.get(3..).unwrap_or("").to_string();

        if path.is_empty() {
            continue;
        }

        // Determine the status type based on git status format
        // First char = index status, second char = working tree status
        let chars: Vec<char> = status_code.chars().collect();
        let status_type = if status_code == "??" {
            StatusType::Untracked
        } else if chars.len() >= 2 {
            let index_status = chars[0];
            let work_status = chars[1];

            if index_status != ' ' && index_status != '?' {
                StatusType::Staged
            } else if work_status != ' ' && work_status != '?' {
                StatusType::Unstaged
            } else {
                StatusType::Unstaged // Default fallback
            }
        } else {
            StatusType::Unstaged
        };

        entries.push(StatusEntry {
            path,
            status_type,
            status_code,
        });
    }

    Ok(entries)
}

/// Returns true if there are any changes (staged, unstaged, or untracked)
pub fn has_changes() -> Result<bool> {
    Ok(status()?.has_changes())
}

/// Returns true if there are any staged changes
pub fn has_staged_changes() -> Result<bool> {
    Ok(status()?.has_staged_changes())
}

/// Returns true if there are any unstaged changes
pub fn has_unstaged_changes() -> Result<bool> {
    Ok(status()?.has_unstaged_changes())
}

/// Returns true if there are any untracked files
pub fn has_untracked_files() -> Result<bool> {
    Ok(status()?.has_untracked())
}

/// Returns true if there are both staged and unstaged changes
pub fn has_mix_changes() -> Result<bool> {
    let s = status()?;
    Ok(s.has_staged_changes() && s.has_unstaged_changes())
}

/// Returns a list of staged files
pub fn get_staged_files() -> Result<Vec<String>> {
    let entries = get_status_entries()?;
    Ok(entries
        .into_iter()
        .filter(|entry| entry.status_type == StatusType::Staged)
        .map(|entry| entry.path)
        .collect())
}

/// Returns a list of unstaged files
pub fn get_unstaged_files() -> Result<Vec<String>> {
    let entries = get_status_entries()?;
    Ok(entries
        .into_iter()
        .filter(|entry| entry.status_type == StatusType::Unstaged)
        .map(|entry| entry.path)
        .collect())
}

/// Returns a list of untracked files
pub fn get_untracked_files() -> Result<Vec<String>> {
    let entries = get_status_entries()?;
    Ok(entries
        .into_iter()
        .filter(|entry| entry.status_type == StatusType::Untracked)
        .map(|entry| entry.path)
        .collect())
}
