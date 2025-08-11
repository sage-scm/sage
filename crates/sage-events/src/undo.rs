use crate::event::{Event, EventData, EventId, ResetMode};
use crate::store::EventStore;
use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UndoError {
    #[error("Cannot undo: {0}")]
    CannotUndo(String),
    #[error("Event not found: {0}")]
    EventNotFound(EventId),
    #[error("No events to undo")]
    NoEventsToUndo,
    #[error("Undo operation not supported for event type")]
    UnsupportedEventType,
    #[error("Store error: {0}")]
    StoreError(#[from] crate::store::EventStoreError),
}

pub struct UndoManager {
    store: EventStore,
}

impl UndoManager {
    pub fn new(store: EventStore) -> Self {
        Self { store }
    }

    pub fn can_undo(&self, event: &Event) -> bool {
        match &event.data {
            EventData::CommitCreated { .. } => true,
            EventData::BranchCreated { .. } => true,
            EventData::BranchDeleted { .. } => true,
            EventData::BranchSwitched { .. } => true,
            EventData::BranchRenamed { .. } => true,
            EventData::CommitAmended { .. } => true,
            EventData::Reset { .. } => true,
            EventData::StashCreated { .. } => true,
            EventData::StashApplied { .. } => true,
            EventData::Push { force, .. } => !force,
            EventData::Merge { fast_forward, .. } => *fast_forward,
            _ => false,
        }
    }

    pub fn undo_last(&self) -> Result<UndoOperation, UndoError> {
        let events = self.store.read_all()?;
        let event = events
            .into_iter()
            .rev()
            .find(|e| self.can_undo(e))
            .ok_or(UndoError::NoEventsToUndo)?;

        self.create_undo_operation(&event)
    }

    pub fn undo_event(&self, event_id: &EventId) -> Result<UndoOperation, UndoError> {
        let event = self
            .store
            .find_by_id(event_id)?
            .ok_or_else(|| UndoError::EventNotFound(event_id.clone()))?;

        if !self.can_undo(&event) {
            return Err(UndoError::CannotUndo(
                "This operation cannot be undone".to_string(),
            ));
        }

        self.create_undo_operation(&event)
    }

    pub fn get_undo_history(&self, limit: usize) -> Result<Vec<(Event, bool)>, UndoError> {
        let events = self.store.get_latest(limit)?;
        
        Ok(events
            .into_iter()
            .map(|e| {
                let can_undo = self.can_undo(&e);
                (e, can_undo)
            })
            .collect())
    }

    fn create_undo_operation(&self, event: &Event) -> Result<UndoOperation, UndoError> {
        match &event.data {
            EventData::CommitCreated { commit_id, branch, .. } => {
                Ok(UndoOperation::ResetBranch {
                    branch: branch.clone(),
                    to_commit: format!("{}~1", commit_id),
                    mode: ResetMode::Mixed,
                })
            }
            EventData::BranchCreated { name, .. } => {
                Ok(UndoOperation::DeleteBranch {
                    name: name.clone(),
                })
            }
            EventData::BranchDeleted { name, last_commit } => {
                Ok(UndoOperation::CreateBranch {
                    name: name.clone(),
                    at_commit: last_commit.clone(),
                })
            }
            EventData::BranchSwitched { from, to: _ } => {
                Ok(UndoOperation::SwitchBranch {
                    to_branch: from.clone(),
                })
            }
            EventData::BranchRenamed { old_name, new_name } => {
                Ok(UndoOperation::RenameBranch {
                    from: new_name.clone(),
                    to: old_name.clone(),
                })
            }
            EventData::CommitAmended { old_commit, branch, .. } => {
                Ok(UndoOperation::ResetBranch {
                    branch: branch.clone(),
                    to_commit: old_commit.clone(),
                    mode: ResetMode::Mixed,
                })
            }
            EventData::Reset { branch, from_commit, mode, .. } => {
                Ok(UndoOperation::ResetBranch {
                    branch: branch.clone(),
                    to_commit: from_commit.clone(),
                    mode: mode.clone(),
                })
            }
            EventData::StashCreated { stash_id, .. } => {
                Ok(UndoOperation::DropStash {
                    stash_id: stash_id.clone(),
                })
            }
            EventData::StashApplied { stash_id: _, branch } => {
                Ok(UndoOperation::ResetBranch {
                    branch: branch.clone(),
                    to_commit: "HEAD~1".to_string(),
                    mode: ResetMode::Hard,
                })
            }
            EventData::Push { branch, commits, .. } => {
                if commits.is_empty() {
                    return Err(UndoError::CannotUndo(
                        "No commits to undo in push".to_string(),
                    ));
                }
                
                Ok(UndoOperation::ResetBranch {
                    branch: branch.clone(),
                    to_commit: format!("HEAD~{}", commits.len()),
                    mode: ResetMode::Mixed,
                })
            }
            EventData::Merge { target_branch, merge_commit, fast_forward, .. } => {
                if *fast_forward {
                    Ok(UndoOperation::ResetBranch {
                        branch: target_branch.clone(),
                        to_commit: format!("{}~1", merge_commit),
                        mode: ResetMode::Hard,
                    })
                } else {
                    Err(UndoError::CannotUndo(
                        "Cannot undo non-fast-forward merge".to_string(),
                    ))
                }
            }
            _ => Err(UndoError::UnsupportedEventType),
        }
    }

    pub fn explain_undo(&self, event: &Event) -> String {
        match &event.data {
            EventData::CommitCreated { message, .. } => {
                format!("Undo commit: {}", message.lines().next().unwrap_or(""))
            }
            EventData::BranchCreated { name, .. } => {
                format!("Delete branch '{}'", name)
            }
            EventData::BranchDeleted { name, .. } => {
                format!("Recreate branch '{}'", name)
            }
            EventData::BranchSwitched { from, to } => {
                format!("Switch back from '{}' to '{}'", to, from)
            }
            EventData::BranchRenamed { old_name, new_name } => {
                format!("Rename branch '{}' back to '{}'", new_name, old_name)
            }
            EventData::CommitAmended { .. } => {
                "Restore original commit before amendment".to_string()
            }
            EventData::Reset { .. } => {
                "Undo reset operation".to_string()
            }
            EventData::StashCreated { .. } => {
                "Drop the created stash".to_string()
            }
            EventData::StashApplied { .. } => {
                "Remove applied stash changes".to_string()
            }
            EventData::Push { commits, .. } => {
                format!("Remove {} pushed commits locally", commits.len())
            }
            EventData::Merge { .. } => {
                "Undo merge operation".to_string()
            }
            _ => "This operation cannot be undone".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UndoOperation {
    ResetBranch {
        branch: String,
        to_commit: String,
        mode: ResetMode,
    },
    CreateBranch {
        name: String,
        at_commit: String,
    },
    DeleteBranch {
        name: String,
    },
    SwitchBranch {
        to_branch: String,
    },
    RenameBranch {
        from: String,
        to: String,
    },
    DropStash {
        stash_id: String,
    },
    CherryPick {
        commit: String,
        onto_branch: String,
    },
    RevertCommit {
        commit: String,
    },
}

impl UndoOperation {
    pub fn describe(&self) -> String {
        match self {
            UndoOperation::ResetBranch { branch, to_commit, mode } => {
                format!("Reset branch '{}' to {} ({:?} mode)", branch, to_commit, mode)
            }
            UndoOperation::CreateBranch { name, at_commit } => {
                format!("Create branch '{}' at commit {}", name, at_commit)
            }
            UndoOperation::DeleteBranch { name } => {
                format!("Delete branch '{}'", name)
            }
            UndoOperation::SwitchBranch { to_branch } => {
                format!("Switch to branch '{}'", to_branch)
            }
            UndoOperation::RenameBranch { from, to } => {
                format!("Rename branch '{}' to '{}'", from, to)
            }
            UndoOperation::DropStash { stash_id } => {
                format!("Drop stash '{}'", stash_id)
            }
            UndoOperation::CherryPick { commit, onto_branch } => {
                format!("Cherry-pick commit {} onto branch '{}'", commit, onto_branch)
            }
            UndoOperation::RevertCommit { commit } => {
                format!("Revert commit {}", commit)
            }
        }
    }
}