// crates/sync-engine/src/protocol.rs
//! Sync protocol definitions

use crate::types::Change;
use serde::{Deserialize, Serialize};

/// Sync request from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Device making the request
    pub device_id: String,
    /// Changes to push to server
    pub changes: Vec<Change>,
    /// Last sync timestamp (to get changes since)
    pub since: Option<chrono::DateTime<chrono::Utc>>,
}

/// Sync response from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Changes from server
    pub changes: Vec<Change>,
    /// Whether sync was successful
    pub success: bool,
    /// Error message if any
    pub error: Option<String>,
}

impl SyncRequest {
    /// Creates a new sync request
    pub fn new(device_id: String, changes: Vec<Change>) -> Self {
        Self {
            device_id,
            changes,
            since: None,
        }
    }

    /// Sets the since timestamp
    pub fn with_since(mut self, since: chrono::DateTime<chrono::Utc>) -> Self {
        self.since = Some(since);
        self
    }
}

impl SyncResponse {
    /// Creates a successful response
    pub fn success(changes: Vec<Change>) -> Self {
        Self {
            changes,
            success: true,
            error: None,
        }
    }

    /// Creates an error response
    pub fn error(message: String) -> Self {
        Self {
            changes: Vec::new(),
            success: false,
            error: Some(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeType, DeviceId, EntityType};

    #[test]
    fn test_sync_request_creation() {
        let device_id = DeviceId::new();
        let change = Change::new(
            device_id.clone(),
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        let request = SyncRequest::new(device_id.to_string(), vec![change]);
        assert_eq!(request.changes.len(), 1);
        assert!(request.since.is_none());
    }

    #[test]
    fn test_sync_request_with_since() {
        let device_id = DeviceId::new();
        let since = chrono::Utc::now();
        let request = SyncRequest::new(device_id.to_string(), vec![]).with_since(since);

        assert!(request.since.is_some());
    }

    #[test]
    fn test_sync_response_success() {
        let response = SyncResponse::success(vec![]);
        assert!(response.success);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_sync_response_error() {
        let response = SyncResponse::error("Network timeout".to_string());
        assert!(!response.success);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Network timeout");
    }

    #[test]
    fn test_serialization() {
        let device_id = DeviceId::new();
        let change = Change::new(
            device_id.clone(),
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        );

        let request = SyncRequest::new(device_id.to_string(), vec![change]);
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: SyncRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.device_id, deserialized.device_id);
    }
}
