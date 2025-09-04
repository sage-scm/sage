#[derive(Debug, Clone)]
pub enum SummaryItem {
    Count(String, usize),
    Changes(usize, usize),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub status: FileStatus,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Success,
    Error,
    Warning,
    Info,
}
