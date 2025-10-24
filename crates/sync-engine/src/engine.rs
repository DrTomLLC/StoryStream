// crates/sync-engine/src/engine.rs
//! Main sync engine

use crate::conflict::ConflictResolver;
use crate::error::{SyncError, SyncResult};
use crate::protocol::{SyncRequest, SyncResponse};
use crate::tracker::ChangeTracker;
use crate::types::{Change, ConflictResolution, DeviceId, SyncState};
use std::sync::{Arc, Mutex};

/// Configuration for the sync engine
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Device identifier
    pub device_id: DeviceId,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
    /// Whether to auto-sync on changes
    pub auto_sync: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            device_id: DeviceId::new(),
            conflict_resolution: ConflictResolution::UseNewest,
            auto_sync: false,
        }
    }
}

/// Main synchronization engine
pub struct SyncEngine {
    config: SyncConfig,
    tracker: ChangeTracker,
    resolver: ConflictResolver,
    state: Arc<Mutex<SyncState>>,
}

impl SyncEngine {
    /// Creates a new sync engine
    pub fn new(config: SyncConfig) -> Self {
        let tracker = ChangeTracker::new(config.device_id.clone());
        let resolver = ConflictResolver::new(config.conflict_resolution);

        Self {
            config,
            tracker,
            resolver,
            state: Arc::new(Mutex::new(SyncState::new())),
        }
    }

    /// Records a local change
    pub fn record_change(
        &self,
        change_type: crate::types::ChangeType,
        entity_type: crate::types::EntityType,
        entity_id: String,
        data: serde_json::Value,
    ) -> SyncResult<()> {
        self.tracker
            .record_change(change_type, entity_type, entity_id, data)?;

        // Update state
        let mut state = self
            .state
            .lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;
        state.pending_changes = self.tracker.pending_count();

        Ok(())
    }

    /// Performs a sync operation
    pub fn sync(&self, remote_changes: Vec<Change>) -> SyncResult<Vec<Change>> {
        // Mark sync as in progress
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;
            if state.in_progress {
                return Err(SyncError::Custom("Sync already in progress".to_string()));
            }
            state.in_progress = true;
        }

        // Get local changes
        let local_changes = self.tracker.pending_changes()?;

        // Detect and resolve conflicts
        let mut resolved_changes = Vec::new();

        for remote in &remote_changes {
            let mut has_conflict = false;

            for local in &local_changes {
                if self.resolver.detect_conflict(local, remote) {
                    has_conflict = true;
                    let conflict_id = self
                        .resolver
                        .record_conflict(local.clone(), remote.clone())?;
                    let winner = self.resolver.auto_resolve(&conflict_id)?;
                    resolved_changes.push(winner);
                    break;
                }
            }

            if !has_conflict {
                resolved_changes.push(remote.clone());
            }
        }

        // Add non-conflicting local changes
        for local in &local_changes {
            let has_conflict = remote_changes
                .iter()
                .any(|remote| self.resolver.detect_conflict(local, remote));

            if !has_conflict {
                resolved_changes.push(local.clone());
            }
        }

        // Clear resolved conflicts
        self.resolver.clear_resolved()?;

        // Clear local changes that were synced
        self.tracker.clear()?;

        // Update state
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;
            state.last_sync = chrono::Utc::now();
            state.pending_changes = 0;
            state.conflicts = self.resolver.unresolved_count();
            state.in_progress = false;
        }

        Ok(resolved_changes)
    }

    /// Creates a sync request with pending changes
    pub fn create_sync_request(&self) -> SyncResult<SyncRequest> {
        let changes = self.tracker.pending_changes()?;
        let state = self
            .state
            .lock()
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))?;

        Ok(
            SyncRequest::new(self.config.device_id.to_string(), changes)
                .with_since(state.last_sync),
        )
    }

    /// Processes a sync response
    pub fn process_sync_response(&self, response: SyncResponse) -> SyncResult<Vec<Change>> {
        if !response.success {
            return Err(SyncError::Network(
                response
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        self.sync(response.changes)
    }

    /// Gets the current sync state
    pub fn state(&self) -> SyncResult<SyncState> {
        self.state
            .lock()
            .map(|s| s.clone())
            .map_err(|_| SyncError::Custom("Lock poisoned".to_string()))
    }

    /// Gets the device ID
    pub fn device_id(&self) -> &DeviceId {
        &self.config.device_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeType, EntityType};

    #[test]
    fn test_engine_creation() {
        let config = SyncConfig::default();
        let _engine = SyncEngine::new(config);
    }

    #[test]
    fn test_record_change() {
        let config = SyncConfig::default();
        let engine = SyncEngine::new(config);

        let result = engine.record_change(
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        assert!(result.is_ok());

        let state = engine.state().unwrap();
        assert_eq!(state.pending_changes, 1);
    }

    #[test]
    fn test_sync_no_conflicts() {
        let config = SyncConfig::default();
        let engine = SyncEngine::new(config);

        // Record local change
        engine
            .record_change(
                ChangeType::Update,
                EntityType::Position,
                "book-1".to_string(),
                serde_json::json!({"position": 1000}),
            )
            .unwrap();

        // Create remote change (different entity)
        let remote_device = DeviceId::new();
        let remote_change = Change::new(
            remote_device,
            ChangeType::Update,
            EntityType::Position,
            "book-2".to_string(),
            serde_json::json!({"position": 2000}),
        );

        let result = engine.sync(vec![remote_change]);
        assert!(result.is_ok());

        let changes = result.unwrap();
        assert_eq!(changes.len(), 2); // Both changes should be included
    }

    #[test]
    fn test_sync_with_conflicts() {
        let config = SyncConfig::default();
        let engine = SyncEngine::new(config);

        // Record local change
        engine
            .record_change(
                ChangeType::Update,
                EntityType::Position,
                "book-123".to_string(),
                serde_json::json!({"position": 1000}),
            )
            .unwrap();

        // Create conflicting remote change
        let remote_device = DeviceId::new();
        let remote_change = Change::new(
            remote_device,
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 2000}),
        );

        let result = engine.sync(vec![remote_change]);
        assert!(result.is_ok());

        let changes = result.unwrap();
        assert_eq!(changes.len(), 1); // Only winner of conflict
    }

    #[test]
    fn test_create_sync_request() {
        let config = SyncConfig::default();
        let engine = SyncEngine::new(config);

        engine
            .record_change(
                ChangeType::Update,
                EntityType::Position,
                "book-123".to_string(),
                serde_json::json!({"position": 1000}),
            )
            .unwrap();

        let request = engine.create_sync_request().unwrap();
        assert_eq!(request.changes.len(), 1);
    }

    #[test]
    fn test_process_sync_response_success() {
        let config = SyncConfig::default();
        let engine = SyncEngine::new(config);

        let response = SyncResponse::success(vec![]);
        let result = engine.process_sync_response(response);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_sync_response_error() {
        let config = SyncConfig::default();
        let engine = SyncEngine::new(config);

        let response = SyncResponse::error("Network error".to_string());
        let result = engine.process_sync_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_concurrent_sync_blocked() {
        let config = SyncConfig::default();
        let engine = SyncEngine::new(config);

        // Start a sync
        {
            let mut state = engine.state.lock().unwrap();
            state.in_progress = true;
        }

        // Try to start another sync
        let result = engine.sync(vec![]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already in progress"));
    }
}
