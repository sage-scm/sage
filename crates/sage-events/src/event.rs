use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub timestamp: DateTime<Utc>,
    pub parent_id: Option<EventId>,
    pub data: EventData,
    pub metadata: EventMetadata,
}

impl Event {
    pub fn new(data: EventData, parent_id: Option<EventId>) -> Self {
        Self {
            id: EventId::new(),
            timestamp: Utc::now(),
            parent_id,
            data,
            metadata: EventMetadata::default(),
        }
    }

    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub user: Option<String>,
    pub session_id: Option<String>,
    pub command: Option<String>,
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self {
            user: None,
            session_id: None,
            command: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EventData {
    CommitCreated {
        commit_id: String,
        message: String,
        files_changed: Vec<String>,
        branch: String,
    },
    BranchCreated {
        name: String,
        from_branch: String,
        commit_id: String,
    },
    BranchDeleted {
        name: String,
        last_commit: String,
    },
    BranchSwitched {
        from: String,
        to: String,
    },
    BranchRenamed {
        old_name: String,
        new_name: String,
    },
    CommitAmended {
        old_commit: String,
        new_commit: String,
        branch: String,
    },
    Rebase {
        branch: String,
        onto: String,
        commits_before: Vec<String>,
        commits_after: Vec<String>,
    },
    CherryPick {
        commit: String,
        from_branch: String,
        to_branch: String,
        new_commit: String,
    },
    Merge {
        source_branch: String,
        target_branch: String,
        merge_commit: String,
        fast_forward: bool,
    },
    Reset {
        branch: String,
        from_commit: String,
        to_commit: String,
        mode: ResetMode,
    },
    StashCreated {
        stash_id: String,
        message: Option<String>,
        branch: String,
    },
    StashApplied {
        stash_id: String,
        branch: String,
    },
    StashDropped {
        stash_id: String,
    },
    Push {
        branch: String,
        remote: String,
        commits: Vec<String>,
        force: bool,
    },
    Pull {
        branch: String,
        remote: String,
        commits_added: Vec<String>,
        merge_required: bool,
    },
    PullRequestCreated {
        branch: String,
        pr_number: u64,
        title: String,
        draft: bool,
    },
    PullRequestUpdated {
        pr_number: u64,
        branch: String,
    },
    WorkspaceChanged {
        files_modified: Vec<String>,
        files_added: Vec<String>,
        files_deleted: Vec<String>,
    },
    ConfigChanged {
        key: String,
        old_value: Option<String>,
        new_value: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResetMode {
    Soft,
    Mixed,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    CommitCreated,
    BranchCreated,
    BranchDeleted,
    BranchSwitched,
    BranchRenamed,
    CommitAmended,
    Rebase,
    CherryPick,
    Merge,
    Reset,
    StashCreated,
    StashApplied,
    StashDropped,
    Push,
    Pull,
    PullRequestCreated,
    PullRequestUpdated,
    WorkspaceChanged,
    ConfigChanged,
}

impl EventData {
    pub fn event_type(&self) -> EventType {
        match self {
            EventData::CommitCreated { .. } => EventType::CommitCreated,
            EventData::BranchCreated { .. } => EventType::BranchCreated,
            EventData::BranchDeleted { .. } => EventType::BranchDeleted,
            EventData::BranchSwitched { .. } => EventType::BranchSwitched,
            EventData::BranchRenamed { .. } => EventType::BranchRenamed,
            EventData::CommitAmended { .. } => EventType::CommitAmended,
            EventData::Rebase { .. } => EventType::Rebase,
            EventData::CherryPick { .. } => EventType::CherryPick,
            EventData::Merge { .. } => EventType::Merge,
            EventData::Reset { .. } => EventType::Reset,
            EventData::StashCreated { .. } => EventType::StashCreated,
            EventData::StashApplied { .. } => EventType::StashApplied,
            EventData::StashDropped { .. } => EventType::StashDropped,
            EventData::Push { .. } => EventType::Push,
            EventData::Pull { .. } => EventType::Pull,
            EventData::PullRequestCreated { .. } => EventType::PullRequestCreated,
            EventData::PullRequestUpdated { .. } => EventType::PullRequestUpdated,
            EventData::WorkspaceChanged { .. } => EventType::WorkspaceChanged,
            EventData::ConfigChanged { .. } => EventType::ConfigChanged,
        }
    }
}