// crates/sync-engine/src/tracker.rs
//! Change tracking for synchronization

use crate::error::{SyncError, SyncResult};
use crate::types::{Change, ChangeType, DeviceId, EntityType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Tracks changes for synchronization
#[derive(Clone)]
pub struct ChangeTracker {
    device_id: DeviceId,
    changes: Arc<Mutex<HashMap<String, Vec<Change>>>>,
}

impl ChangeTracker {
    /// Creates a new change tracker
    pub fn new(device_id: DeviceId) -> Self {
        Self {
            device_id,
            changes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Records a change
    pub fn record_change(
        &self,
        change_type: ChangeType,
        entity_type: EntityType,
        entity_id: String,
        data: serde_json::Value,
    ) -> SyncResult<()> {
        let change = Change::new(
            self.device_id.clone(),
            change_type,
            entity_type,
            entity_id.clone(),
            data,
        );

        let mut changes = self
            .changes
            .lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;

        changes
            .entry(entity_id)
            .or_insert_with(Vec::new)
            .push(change);

        Ok(())
    }

    /// Gets all pending changes
    pub fn pending_changes(&self) -> SyncResult<Vec<Change>> {
        let changes = self
            .changes
            .lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;

        let mut all_changes = Vec::new();
        for changes_list in changes.values() {
            all_changes.extend(changes_list.iter().cloned());
        }

        // Sort by timestamp
        all_changes.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(all_changes)
    }

    /// Gets changes for a specific entity
    pub fn changes_for_entity(&self, entity_id: &str) -> SyncResult<Vec<Change>> {
        let changes = self
            .changes
            .lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;

        Ok(changes.get(entity_id).cloned().unwrap_or_default())
    }

    /// Clears all tracked changes
    pub fn clear(&self) -> SyncResult<()> {
        let mut changes = self
            .changes
            .lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;

        changes.clear();
        Ok(())
    }

    /// Removes changes for a specific entity
    pub fn clear_entity(&self, entity_id: &str) -> SyncResult<()> {
        let mut changes = self
            .changes
            .lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;

        changes.remove(entity_id);
        Ok(())
    }

    /// Returns the number of pending changes
    pub fn pending_count(&self) -> usize {
        self.changes
            .lock()
            .map(|c| c.values().map(|v| v.len()).sum())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let device_id = DeviceId::new();
        let tracker = ChangeTracker::new(device_id);
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn test_record_change() {
        let device_id = DeviceId::new();
        let tracker = ChangeTracker::new(device_id);

        let result = tracker.record_change(
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        assert!(result.is_ok());
        assert_eq!(tracker.pending_count(), 1);
    }

    #[test]
    fn test_pending_changes() {
        let device_id = DeviceId::new();
        let tracker = ChangeTracker::new(device_id);

        tracker
            .record_change(
                ChangeType::Update,
                EntityType::Position,
                "book-1".to_string(),
                serde_json::json!({"position": 1000}),
            )
            .unwrap();

        tracker
            .record_change(
                ChangeType::Update,
                EntityType::Bookmark,
                "bookmark-1".to_string(),
                serde_json::json!({"title": "Important"}),
            )
            .unwrap();

        let changes = tracker.pending_changes().unwrap();
        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn test_changes_for_entity() {
        let device_id = DeviceId::new();
        let tracker = ChangeTracker::new(device_id);

        tracker
            .record_change(
                ChangeType::Update,
                EntityType::Position,
                "book-123".to_string(),
                serde_json::json!({"position": 1000}),
            )
            .unwrap();

        tracker
            .record_change(
                ChangeType::Update,
                EntityType::Position,
                "book-123".to_string(),
                serde_json::json!({"position": 2000}),
            )
            .unwrap();

        let changes = tracker.changes_for_entity("book-123").unwrap();
        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn test_clear_changes() {
        let device_id = DeviceId::new();
        let tracker = ChangeTracker::new(device_id);

        tracker
            .record_change(
                ChangeType::Update,
                EntityType::Position,
                "book-123".to_string(),
                serde_json::json!({"position": 1000}),
            )
            .unwrap();

        assert_eq!(tracker.pending_count(), 1);

        tracker.clear().unwrap();
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn test_clear_entity() {
        let device_id = DeviceId::new();
        let tracker = ChangeTracker::new(device_id);

        tracker
            .record_change(
                ChangeType::Update,
                EntityType::Position,
                "book-1".to_string(),
                serde_json::json!({"position": 1000}),
            )
            .unwrap();

        tracker
            .record_change(
                ChangeType::Update,
                EntityType::Position,
                "book-2".to_string(),
                serde_json::json!({"position": 2000}),
            )
            .unwrap();

        assert_eq!(tracker.pending_count(), 2);

        tracker.clear_entity("book-1").unwrap();
        assert_eq!(tracker.pending_count(), 1);
    }
}
