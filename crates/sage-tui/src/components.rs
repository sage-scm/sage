/// Items that can appear in a summary line
#[derive(Debug, Clone)]
pub enum SummaryItem {
    /// A count with label (e.g., "3 files")
    Count(String, usize),
    /// Addition/deletion counts
    Changes(usize, usize),
    /// Raw text
    Text(String),
}

/// Represents a file change
#[derive(Debug, Clone)]
pub struct FileChange {
    /// File path
    pub path: String,
    /// Lines added
    pub additions: usize,
    /// Lines deleted
    pub deletions: usize,
    /// File status
    pub status: FileStatus,
    /// Optional description
    pub description: Option<String>,
}

impl FileChange {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            additions: 0,
            deletions: 0,
            status: FileStatus::Modified,
            description: None,
        }
    }

    pub fn added(mut self, lines: usize) -> Self {
        self.additions = lines;
        self.status = FileStatus::Added;
        self
    }

    pub fn deleted(mut self, lines: usize) -> Self {
        self.deletions = lines;
        self.status = FileStatus::Deleted;
        self
    }

    pub fn modified(mut self, additions: usize, deletions: usize) -> Self {
        self.additions = additions;
        self.deletions = deletions;
        self.status = FileStatus::Modified;
        self
    }

    pub fn renamed(mut self) -> Self {
        self.status = FileStatus::Renamed;
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// File status in version control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
}

/// Message type for colored output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Success,
    Error,
    Warning,
    Info,
}
