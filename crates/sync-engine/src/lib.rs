// crates/sync-engine/src/lib.rs
//! Cross-device synchronization engine
//!
//! This module provides synchronization capabilities for sharing data across multiple devices:
//! - Playback position syncing
//! - Bookmark synchronization
//! - Library metadata syncing
//! - Conflict detection and resolution
//!
//! # Example
//!
//! ```rust
//! use storystream_sync_engine::{SyncEngine, SyncConfig, ConflictResolution};
//!
//! let config = SyncConfig {
//!     device_id: storystream_sync_engine::DeviceId::new(),
//!     conflict_resolution: ConflictResolution::UseNewest,
//!     auto_sync: false,
//! };
//!
//! let engine = SyncEngine::new(config);
//!
//! // Record a change
//! engine.record_change(
//!     storystream_sync_engine::ChangeType::Update,
//!     storystream_sync_engine::EntityType::Position,
//!     "book-123".to_string(),
//!     serde_json::json!({"position": 1000}),
//! ).unwrap();
//! ```

mod conflict;
mod engine;
mod error;
mod protocol;
mod tracker;
mod types;

pub use conflict::ConflictResolver;
pub use engine::{SyncConfig, SyncEngine};
pub use error::{SyncError, SyncResult};
pub use protocol::{SyncRequest, SyncResponse};
pub use tracker::ChangeTracker;
pub use types::{
    Change, ChangeType, Conflict, ConflictResolution, DeviceId, EntityType, SyncState,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_exports_accessible() {
        // Verify all types are exported
        let _: DeviceId = DeviceId::new();
        let _: SyncConfig = SyncConfig::default();
        let config = SyncConfig::default();
        let _: SyncEngine = SyncEngine::new(config);
    }
}