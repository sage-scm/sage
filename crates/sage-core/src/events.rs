use anyhow::Result;
use sage_events::undo::UndoOperation;
use sage_events::{Event, EventData, EventId, EventStore, UndoManager};
use std::path::Path;

pub struct EventManager {
    store: EventStore,
}

impl EventManager {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let store = EventStore::new(repo_path)?;

        Ok(Self { store })
    }

    pub fn track(&self, data: EventData) -> Result<()> {
        let event = Event::new(data, self.get_latest_id()?);
        self.store.append(&event)?;
        Ok(())
    }

    pub fn track_with_parent(&self, data: EventData, parent_id: EventId) -> Result<()> {
        let event = Event::new(data, Some(parent_id));
        self.store.append(&event)?;
        Ok(())
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<Event>> {
        Ok(self.store.get_latest(limit)?)
    }

    pub fn get_undo_history(&self, limit: usize) -> Result<Vec<(Event, bool)>> {
        // Get the parent of parent (go from .git/sage/events.jsonl to repo root)
        let snapshot = self.store.snapshot()?;
        let repo_path = snapshot
            .path
            .parent() // .git/sage
            .and_then(|p| p.parent()) // .git
            .and_then(|p| p.parent()) // repo root
            .ok_or_else(|| anyhow::anyhow!("Cannot determine repository path"))?;

        let undo_manager = UndoManager::new(EventStore::new(repo_path)?);
        Ok(undo_manager.get_undo_history(limit)?)
    }

    pub fn undo_last(&self) -> Result<UndoOperation> {
        // Get the parent of parent (go from .git/sage/events.jsonl to repo root)
        let snapshot = self.store.snapshot()?;
        let repo_path = snapshot
            .path
            .parent() // .git/sage
            .and_then(|p| p.parent()) // .git
            .and_then(|p| p.parent()) // repo root
            .ok_or_else(|| anyhow::anyhow!("Cannot determine repository path"))?;

        let undo_manager = UndoManager::new(EventStore::new(repo_path)?);
        Ok(undo_manager.undo_last()?)
    }

    pub fn undo_event(&self, event_id: &EventId) -> Result<UndoOperation> {
        // Get the parent of parent (go from .git/sage/events.jsonl to repo root)
        let snapshot = self.store.snapshot()?;
        let repo_path = snapshot
            .path
            .parent() // .git/sage
            .and_then(|p| p.parent()) // .git
            .and_then(|p| p.parent()) // repo root
            .ok_or_else(|| anyhow::anyhow!("Cannot determine repository path"))?;

        let undo_manager = UndoManager::new(EventStore::new(repo_path)?);
        Ok(undo_manager.undo_event(event_id)?)
    }

    pub fn explain_undo(&self, event: &Event) -> String {
        // Get the parent of parent (go from .git/sage/events.jsonl to repo root)
        let snapshot = self.store.snapshot().unwrap();
        let repo_path = snapshot
            .path
            .parent() // .git/sage
            .and_then(|p| p.parent()) // .git
            .and_then(|p| p.parent()) // repo root
            .unwrap();

        let undo_manager = UndoManager::new(EventStore::new(repo_path).unwrap());
        undo_manager.explain_undo(event)
    }

    fn get_latest_id(&self) -> Result<Option<EventId>> {
        let events = self.store.get_latest(1)?;
        Ok(events.first().map(|e| e.id.clone()))
    }
}
