// crates/sync-engine/src/conflict.rs
//! Conflict detection and resolution

use crate::error::{SyncError, SyncResult};
use crate::types::{Change, Conflict, ConflictResolution};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Manages conflict detection and resolution
#[derive(Clone)]
pub struct ConflictResolver {
    conflicts: Arc<Mutex<HashMap<String, Conflict>>>,
    default_strategy: ConflictResolution,
}

impl ConflictResolver {
    /// Creates a new conflict resolver with a default strategy
    pub fn new(default_strategy: ConflictResolution) -> Self {
        Self {
            conflicts: Arc::new(Mutex::new(HashMap::new())),
            default_strategy,
        }
    }

    /// Detects if two changes conflict
    pub fn detect_conflict(&self, local: &Change, remote: &Change) -> bool {
        // Changes conflict if they affect the same entity
        local.entity_id == remote.entity_id &&
            local.entity_type == remote.entity_type &&
            local.device_id != remote.device_id
    }

    /// Records a conflict
    pub fn record_conflict(&self, local: Change, remote: Change) -> SyncResult<String> {
        let conflict = Conflict::new(local, remote);
        let conflict_id = conflict.id.clone();

        let mut conflicts = self.conflicts.lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;

        conflicts.insert(conflict_id.clone(), conflict);

        Ok(conflict_id)
    }

    /// Resolves a conflict using the default strategy
    pub fn auto_resolve(&self, conflict_id: &str) -> SyncResult<Change> {
        let mut conflicts = self.conflicts.lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;

        let conflict = conflicts.get_mut(conflict_id)
            .ok_or_else(|| SyncError::Custom("Conflict not found".to_string()))?;

        conflict.resolve(self.default_strategy);

        // Return the winning change based on strategy
        let winner = match self.default_strategy {
            ConflictResolution::UseLocal => conflict.local.clone(),
            ConflictResolution::UseRemote => conflict.remote.clone(),
            ConflictResolution::UseNewest => {
                if conflict.local.is_newer_than(&conflict.remote) {
                    conflict.local.clone()
                } else {
                    conflict.remote.clone()
                }
            }
            ConflictResolution::Merge => {
                // For simple merge, use the newest
                if conflict.local.is_newer_than(&conflict.remote) {
                    conflict.local.clone()
                } else {
                    conflict.remote.clone()
                }
            }
        };

        Ok(winner)
    }

    /// Gets all unresolved conflicts
    pub fn unresolved_conflicts(&self) -> SyncResult<Vec<Conflict>> {
        let conflicts = self.conflicts.lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;

        Ok(conflicts.values()
            .filter(|c| !c.is_resolved())
            .cloned()
            .collect())
    }

    /// Gets the number of unresolved conflicts
    pub fn unresolved_count(&self) -> usize {
        self.conflicts.lock()
            .map(|c| c.values().filter(|conf| !conf.is_resolved()).count())
            .unwrap_or(0)
    }

    /// Clears all resolved conflicts
    pub fn clear_resolved(&self) -> SyncResult<()> {
        let mut conflicts = self.conflicts.lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;

        conflicts.retain(|_, conflict| !conflict.is_resolved());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeType, DeviceId, EntityType};

    #[test]
    fn test_resolver_creation() {
        let resolver = ConflictResolver::new(ConflictResolution::UseNewest);
        assert_eq!(resolver.unresolved_count(), 0);
    }

    #[test]
    fn test_detect_conflict() {
        let resolver = ConflictResolver::new(ConflictResolution::UseNewest);

        let device1 = DeviceId::new();
        let device2 = DeviceId::new();

        let change1 = Change::new(
            device1,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        let change2 = Change::new(
            device2,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 2000}),
        );

        assert!(resolver.detect_conflict(&change1, &change2));
    }

    #[test]
    fn test_no_conflict_same_device() {
        let resolver = ConflictResolver::new(ConflictResolution::UseNewest);

        let device = DeviceId::new();

        let change1 = Change::new(
            device.clone(),
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        let change2 = Change::new(
            device,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 2000}),
        );

        assert!(!resolver.detect_conflict(&change1, &change2));
    }

    #[test]
    fn test_record_conflict() {
        let resolver = ConflictResolver::new(ConflictResolution::UseNewest);

        let device1 = DeviceId::new();
        let device2 = DeviceId::new();

        let local = Change::new(
            device1,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        let remote = Change::new(
            device2,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 2000}),
        );

        let conflict_id = resolver.record_conflict(local, remote).unwrap();
        assert!(!conflict_id.is_empty());
        assert_eq!(resolver.unresolved_count(), 1);
    }

    #[test]
    fn test_auto_resolve_newest() {
        let resolver = ConflictResolver::new(ConflictResolution::UseNewest);

        let device1 = DeviceId::new();
        let device2 = DeviceId::new();

        let local = Change::new(
            device1,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        std::thread::sleep(std::time::Duration::from_millis(10));

        let remote = Change::new(
            device2,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 2000}),
        );

        let conflict_id = resolver.record_conflict(local, remote.clone()).unwrap();
        let winner = resolver.auto_resolve(&conflict_id).unwrap();

        // Remote should win because it's newer
        assert_eq!(winner.data, remote.data);
    }

    #[test]
    fn test_clear_resolved() {
        let resolver = ConflictResolver::new(ConflictResolution::UseNewest);

        let device1 = DeviceId::new();
        let device2 = DeviceId::new();

        let local = Change::new(
            device1,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        let remote = Change::new(
            device2,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 2000}),
        );

        let conflict_id = resolver.record_conflict(local, remote).unwrap();
        assert_eq!(resolver.unresolved_count(), 1);

        resolver.auto_resolve(&conflict_id).unwrap();
        resolver.clear_resolved().unwrap();

        assert_eq!(resolver.unresolved_count(), 0);
    }
}