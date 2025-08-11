use crate::event::{Event, EventData, EventId, EventType};
use anyhow::Result;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Event not found: {0}")]
    EventNotFound(EventId),
    #[error("Invalid event log")]
    InvalidEventLog,
}

pub struct EventStore {
    path: PathBuf,
    max_events: usize,
}

impl EventStore {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let git_dir = repo_path.join(".git");
        if !git_dir.exists() || !git_dir.is_dir() {
            return Err(anyhow::anyhow!("Not in a git repository"));
        }
        
        let sage_dir = git_dir.join("sage");
        fs::create_dir_all(&sage_dir)?;
        
        Ok(Self {
            path: sage_dir.join("events.jsonl"),
            max_events: 10000,
        })
    }

    pub fn append(&self, event: &Event) -> Result<(), EventStoreError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        
        let json = serde_json::to_string(event)?;
        writeln!(file, "{}", json)?;
        
        self.compact_if_needed()?;
        
        Ok(())
    }

    pub fn read_all(&self) -> Result<Vec<Event>, EventStoreError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            
            let event: Event = serde_json::from_str(&line)?;
            events.push(event);
        }

        Ok(events)
    }

    pub fn find_by_id(&self, id: &EventId) -> Result<Option<Event>, EventStoreError> {
        let events = self.read_all()?;
        Ok(events.into_iter().find(|e| &e.id == id))
    }

    pub fn find_by_type(&self, event_type: EventType) -> Result<Vec<Event>, EventStoreError> {
        let events = self.read_all()?;
        Ok(events
            .into_iter()
            .filter(|e| e.data.event_type() == event_type)
            .collect())
    }

    pub fn get_latest(&self, count: usize) -> Result<Vec<Event>, EventStoreError> {
        let events = self.read_all()?;
        let len = events.len();
        let start = len.saturating_sub(count);
        Ok(events[start..].to_vec())
    }

    pub fn get_since(&self, event_id: &EventId) -> Result<Vec<Event>, EventStoreError> {
        let events = self.read_all()?;
        
        let position = events
            .iter()
            .position(|e| &e.id == event_id)
            .ok_or(EventStoreError::EventNotFound(event_id.clone()))?;
        
        Ok(events[(position + 1)..].to_vec())
    }

    pub fn get_until(&self, event_id: &EventId) -> Result<Vec<Event>, EventStoreError> {
        let events = self.read_all()?;
        
        let position = events
            .iter()
            .position(|e| &e.id == event_id)
            .ok_or(EventStoreError::EventNotFound(event_id.clone()))?;
        
        Ok(events[..=position].to_vec())
    }

    pub fn clear(&self) -> Result<(), EventStoreError> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }

    fn compact_if_needed(&self) -> Result<(), EventStoreError> {
        let events = self.read_all()?;
        
        if events.len() > self.max_events {
            let keep_count = self.max_events * 3 / 4;
            let skip_count = events.len() - keep_count;
            let events_to_keep: Vec<_> = events
                .into_iter()
                .skip(skip_count)
                .collect();
            
            self.rewrite(events_to_keep)?;
        }
        
        Ok(())
    }

    fn rewrite(&self, events: Vec<Event>) -> Result<(), EventStoreError> {
        let temp_path = self.path.with_extension("tmp");
        
        {
            let mut file = File::create(&temp_path)?;
            for event in &events {
                let json = serde_json::to_string(event)?;
                writeln!(file, "{}", json)?;
            }
            file.sync_all()?;
        }
        
        fs::rename(&temp_path, &self.path)?;
        
        Ok(())
    }

    pub fn get_branch_history(&self, branch_name: &str) -> Result<Vec<Event>, EventStoreError> {
        let events = self.read_all()?;
        
        Ok(events
            .into_iter()
            .filter(|event| match &event.data {
                EventData::CommitCreated { branch, .. } => branch == branch_name,
                EventData::CommitAmended { branch, .. } => branch == branch_name,
                EventData::BranchSwitched { to, from } => to == branch_name || from == branch_name,
                EventData::Rebase { branch, .. } => branch == branch_name,
                EventData::Merge { target_branch, .. } => target_branch == branch_name,
                EventData::Reset { branch, .. } => branch == branch_name,
                EventData::Push { branch, .. } => branch == branch_name,
                EventData::Pull { branch, .. } => branch == branch_name,
                _ => false,
            })
            .collect())
    }

    pub fn snapshot(&self) -> Result<EventSnapshot, EventStoreError> {
        let events = self.read_all()?;
        let latest_id = events.last().map(|e| e.id.clone());
        
        Ok(EventSnapshot {
            event_count: events.len(),
            latest_event_id: latest_id,
            path: self.path.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct EventSnapshot {
    pub event_count: usize,
    pub latest_event_id: Option<EventId>,
    pub path: PathBuf,
}