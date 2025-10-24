// crates/sync-engine/src/error.rs
//! Error types for sync operations

use thiserror::Error;

/// Result type for sync operations
pub type SyncResult<T> = Result<T, SyncError>;

/// Errors that can occur during synchronization
#[derive(Debug, Error)]
pub enum SyncError {
    /// Conflict detected during sync
    #[error("Sync conflict: {0}")]
    Conflict(String),

    /// Invalid sync data
    #[error("Invalid sync data: {0}")]
    InvalidData(String),

    /// Sync not initialized
    #[error("Sync engine not initialized")]
    NotInitialized,

    /// Device not registered
    #[error("Device not registered: {0}")]
    DeviceNotRegistered(String),

    /// Network error during sync
    #[error("Network error: {0}")]
    Network(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SyncError::Conflict("version mismatch".to_string());
        assert!(err.to_string().contains("Sync conflict"));
    }

    #[test]
    fn test_invalid_data_error() {
        let err = SyncError::InvalidData("corrupted".to_string());
        assert!(err.to_string().contains("Invalid sync data"));
    }

    #[test]
    fn test_not_initialized_error() {
        let err = SyncError::NotInitialized;
        assert!(err.to_string().contains("not initialized"));
    }

    #[test]
    fn test_device_not_registered_error() {
        let err = SyncError::DeviceNotRegistered("device-123".to_string());
        assert!(err.to_string().contains("device-123"));
    }
}
