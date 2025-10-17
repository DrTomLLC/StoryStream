// crates/sync-engine/src/types.rs
//! Core sync types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique device identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(String);

impl DeviceId {
    /// Creates a new device ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Creates a device ID from a string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Returns the device ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for DeviceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of change that occurred
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Item was created
    Create,
    /// Item was updated
    Update,
    /// Item was deleted
    Delete,
}

/// Entity type being synced
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    /// Playback position
    Position,
    /// Bookmark
    Bookmark,
    /// Book metadata
    Book,
    /// Configuration setting
    Setting,
}

/// A tracked change to sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Unique change ID
    pub id: String,
    /// Device that made the change
    pub device_id: DeviceId,
    /// Type of change
    pub change_type: ChangeType,
    /// Entity type
    pub entity_type: EntityType,
    /// Entity ID
    pub entity_id: String,
    /// Change data (JSON)
    pub data: serde_json::Value,
    /// Timestamp when change occurred
    pub timestamp: DateTime<Utc>,
    /// Version number for conflict resolution
    pub version: u64,
}

impl Change {
    /// Creates a new change
    pub fn new(
        device_id: DeviceId,
        change_type: ChangeType,
        entity_type: EntityType,
        entity_id: String,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            device_id,
            change_type,
            entity_type,
            entity_id,
            data,
            timestamp: Utc::now(),
            version: 1,
        }
    }

    /// Returns true if this is a deletion
    pub fn is_delete(&self) -> bool {
        matches!(self.change_type, ChangeType::Delete)
    }

    /// Returns true if this change is newer than another
    pub fn is_newer_than(&self, other: &Change) -> bool {
        self.timestamp > other.timestamp
    }
}

/// Sync state for tracking progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// Last sync timestamp
    pub last_sync: DateTime<Utc>,
    /// Number of changes pending upload
    pub pending_changes: usize,
    /// Number of conflicts
    pub conflicts: usize,
    /// Whether sync is currently in progress
    pub in_progress: bool,
}

impl SyncState {
    /// Creates a new sync state
    pub fn new() -> Self {
        Self {
            last_sync: Utc::now(),
            pending_changes: 0,
            conflicts: 0,
            in_progress: false,
        }
    }

    /// Returns true if there are pending changes
    pub fn has_pending_changes(&self) -> bool {
        self.pending_changes > 0
    }

    /// Returns true if there are conflicts
    pub fn has_conflicts(&self) -> bool {
        self.conflicts > 0
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Use the local version
    UseLocal,
    /// Use the remote version
    UseRemote,
    /// Use the newest version (by timestamp)
    UseNewest,
    /// Merge both versions (if possible)
    Merge,
}

/// A detected conflict between local and remote changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Conflict ID
    pub id: String,
    /// Local change
    pub local: Change,
    /// Remote change
    pub remote: Change,
    /// When conflict was detected
    pub detected_at: DateTime<Utc>,
    /// Resolution strategy (if resolved)
    pub resolution: Option<ConflictResolution>,
}

impl Conflict {
    /// Creates a new conflict
    pub fn new(local: Change, remote: Change) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            local,
            remote,
            detected_at: Utc::now(),
            resolution: None,
        }
    }

    /// Returns true if conflict is resolved
    pub fn is_resolved(&self) -> bool {
        self.resolution.is_some()
    }

    /// Resolves the conflict using a strategy
    pub fn resolve(&mut self, strategy: ConflictResolution) {
        self.resolution = Some(strategy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_id_creation() {
        let id1 = DeviceId::new();
        let id2 = DeviceId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_device_id_from_string() {
        let id = DeviceId::from_string("test-device".to_string());
        assert_eq!(id.as_str(), "test-device");
    }

    #[test]
    fn test_change_creation() {
        let device_id = DeviceId::new();
        let change = Change::new(
            device_id,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        assert_eq!(change.change_type, ChangeType::Update);
        assert_eq!(change.entity_type, EntityType::Position);
        assert!(!change.is_delete());
    }

    #[test]
    fn test_change_is_newer() {
        let device_id = DeviceId::new();
        let change1 = Change::new(
            device_id.clone(),
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        std::thread::sleep(std::time::Duration::from_millis(10));

        let change2 = Change::new(
            device_id,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 2000}),
        );

        assert!(change2.is_newer_than(&change1));
        assert!(!change1.is_newer_than(&change2));
    }

    #[test]
    fn test_sync_state_default() {
        let state = SyncState::new();
        assert_eq!(state.pending_changes, 0);
        assert_eq!(state.conflicts, 0);
        assert!(!state.in_progress);
        assert!(!state.has_pending_changes());
        assert!(!state.has_conflicts());
    }

    #[test]
    fn test_conflict_creation() {
        let device_id = DeviceId::new();
        let local = Change::new(
            device_id.clone(),
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        let remote = Change::new(
            device_id,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 2000}),
        );

        let mut conflict = Conflict::new(local, remote);
        assert!(!conflict.is_resolved());

        conflict.resolve(ConflictResolution::UseNewest);
        assert!(conflict.is_resolved());
    }

    #[test]
    fn test_change_type_equality() {
        assert_eq!(ChangeType::Create, ChangeType::Create);
        assert_ne!(ChangeType::Create, ChangeType::Update);
    }

    #[test]
    fn test_entity_type_equality() {
        assert_eq!(EntityType::Position, EntityType::Position);
        assert_ne!(EntityType::Position, EntityType::Bookmark);
    }
}