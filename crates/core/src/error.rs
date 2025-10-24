//! Error types and recovery strategies for StoryStream
//!
//! This module provides a comprehensive error handling system with three severity tiers:
//! - **Recoverable**: Can be automatically retried (network timeouts, etc.)
//! - **Degraded**: Feature disabled but app continues (missing codec, etc.)
//! - **Fatal**: Requires app restart or user intervention (corrupted database, etc.)
//!
//! Each error includes a recovery action to guide automatic error handling.

use std::fmt;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Recovery actions that can be taken when an error occurs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Retry the operation immediately (e.g., transient network glitch)
    RetryImmediate,
    /// Retry with exponential backoff (e.g., server temporarily unavailable)
    RetryWithBackoff,
    /// Disable the failing feature and continue (e.g., missing optional codec)
    DisableFeature,
    /// Attempt to repair the database and retry
    RepairDatabase,
    /// Restore from the most recent backup
    RestoreBackup,
    /// Perform a safe shutdown and require user restart
    SafeShutdown,
    /// No automatic recovery - user intervention required
    UserIntervention,
}

impl fmt::Display for RecoveryAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RetryImmediate => write!(f, "Retrying immediately"),
            Self::RetryWithBackoff => write!(f, "Retrying with backoff"),
            Self::DisableFeature => write!(f, "Disabling feature"),
            Self::RepairDatabase => write!(f, "Repairing database"),
            Self::RestoreBackup => write!(f, "Restoring from backup"),
            Self::SafeShutdown => write!(f, "Performing safe shutdown"),
            Self::UserIntervention => write!(f, "User intervention required"),
        }
    }
}

/// Error severity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Error can be automatically recovered from
    Recoverable,
    /// Feature degraded but app can continue
    Degraded,
    /// Critical error requiring restart or user action
    Fatal,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recoverable => write!(f, "Recoverable"),
            Self::Degraded => write!(f, "Degraded"),
            Self::Fatal => write!(f, "Fatal"),
        }
    }
}

/// Main error type for StoryStream
#[derive(Error, Debug)]
pub enum AppError {
    // ===== Network Errors =====
    /// Network request failed
    #[error("Network error: {message}")]
    NetworkError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Network timeout
    #[error("Network timeout after {seconds}s: {operation}")]
    NetworkTimeout { operation: String, seconds: u64 },

    /// Connection lost during operation
    #[error("Connection lost: {message}")]
    ConnectionLost { message: String },

    /// Invalid URL provided
    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    // ===== Database Errors =====
    /// Database operation failed
    #[error("Database error: {message}")]
    DatabaseError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Database is corrupted and needs repair
    #[error("Database corrupted: {details}")]
    DatabaseCorrupted { details: String },

    /// Database migration failed
    #[error("Migration failed: {version} - {reason}")]
    MigrationFailed { version: String, reason: String },

    /// Database is locked by another process
    #[error("Database locked: {operation}")]
    DatabaseLocked { operation: String },

    /// Record not found in database
    #[error("Record not found: {entity} with {identifier}")]
    RecordNotFound { entity: String, identifier: String },

    // ===== Audio/Media Errors =====
    /// Unsupported audio format
    #[error("Unsupported audio format: {format} in file {file}")]
    UnsupportedFormat { format: String, file: PathBuf },

    /// Audio decoding failed
    #[error("Audio decode error: {message}")]
    AudioDecodeError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Corrupted audio file
    #[error("Corrupted audio file: {file} - {reason}")]
    CorruptedAudioFile { file: PathBuf, reason: String },

    /// Audio playback device error
    #[error("Playback device error: {message}")]
    PlaybackDeviceError { message: String },

    /// Invalid audio position (seeking)
    #[error("Invalid audio position: {position}ms (file duration: {duration}ms)")]
    InvalidPosition { position: u64, duration: u64 },

    // ===== File System Errors =====
    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Permission denied for file operation
    #[error("Permission denied: {operation} on {path}")]
    PermissionDenied { operation: String, path: PathBuf },

    /// Disk full - cannot write
    #[error("Disk full: needed {needed_bytes} bytes, available {available_bytes} bytes")]
    DiskFull {
        needed_bytes: u64,
        available_bytes: u64,
    },

    /// General I/O error
    #[error("I/O error: {message}")]
    IoError {
        message: String,
        #[source]
        source: io::Error,
    },

    // ===== Parsing/Metadata Errors =====
    /// Failed to parse metadata
    #[error("Metadata parse error in {file}: {reason}")]
    MetadataParseError { file: PathBuf, reason: String },

    /// Invalid metadata format
    #[error("Invalid metadata: {field} has invalid value '{value}'")]
    InvalidMetadata { field: String, value: String },

    /// Missing required metadata
    #[error("Missing required metadata: {field} in {file}")]
    MissingMetadata { field: String, file: PathBuf },

    // ===== Content Source Errors =====
    /// Content source (LibriVox, Archive) unavailable
    #[error("Content source '{provider}' unavailable: {reason}")]
    ContentSourceUnavailable { provider: String, reason: String },

    /// Invalid response from content source
    #[error("Invalid response from {provider}: {details}")]
    InvalidContentResponse { provider: String, details: String },

    /// Content not found at source
    #[error("Content not found: {identifier} at {provider}")]
    ContentNotFound {
        identifier: String,
        provider: String,
    },

    // ===== Configuration/Settings Errors =====
    /// Invalid configuration
    #[error("Invalid configuration: {setting} = '{value}' ({reason})")]
    InvalidConfiguration {
        setting: String,
        value: String,
        reason: String,
    },

    /// Configuration file corrupted
    #[error("Configuration corrupted: {path}")]
    ConfigurationCorrupted { path: PathBuf },

    // ===== Sync Errors (Phase 3 prep) =====
    /// Sync conflict detected
    #[error("Sync conflict: {entity} modified on multiple devices")]
    SyncConflict { entity: String },

    /// Sync authentication failed
    #[error("Sync authentication failed: {provider}")]
    SyncAuthFailed { provider: String },

    // ===== Cache Errors =====
    /// Cache write failed
    #[error("Cache write failed: {reason}")]
    CacheWriteFailed { reason: String },

    /// Cache corrupted
    #[error("Cache corrupted at {path}: {reason}")]
    CacheCorrupted { path: PathBuf, reason: String },

    // ===== System Resource Errors =====
    /// Out of memory
    #[error("Out of memory: requested {requested_bytes} bytes")]
    OutOfMemory { requested_bytes: u64 },

    /// Too many open files
    #[error("Too many open files (limit: {limit})")]
    TooManyOpenFiles { limit: usize },

    // ===== Generic Errors =====
    /// Generic internal error
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Operation cancelled by user
    #[error("Operation cancelled: {operation}")]
    Cancelled { operation: String },

    /// Invalid argument provided
    #[error("Invalid argument: {argument} - {reason}")]
    InvalidArgument { argument: String, reason: String },
}

impl AppError {
    /// Returns the severity level of this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Recoverable - can retry automatically
            Self::NetworkError { .. }
            | Self::NetworkTimeout { .. }
            | Self::ConnectionLost { .. }
            | Self::DatabaseLocked { .. }
            | Self::CacheWriteFailed { .. } => ErrorSeverity::Recoverable,

            // Degraded - disable feature but continue
            Self::UnsupportedFormat { .. }
            | Self::ContentSourceUnavailable { .. }
            | Self::ContentNotFound { .. }
            | Self::PlaybackDeviceError { .. }
            | Self::InvalidContentResponse { .. }
            | Self::CacheCorrupted { .. }
            | Self::SyncConflict { .. }
            | Self::SyncAuthFailed { .. } => ErrorSeverity::Degraded,

            // Fatal - requires restart or user action
            Self::DatabaseCorrupted { .. }
            | Self::MigrationFailed { .. }
            | Self::OutOfMemory { .. }
            | Self::DiskFull { .. }
            | Self::ConfigurationCorrupted { .. }
            | Self::CorruptedAudioFile { .. }
            | Self::TooManyOpenFiles { .. } => ErrorSeverity::Fatal,

            // Context-dependent - default to degraded
            _ => ErrorSeverity::Degraded,
        }
    }

    /// Returns the recommended recovery action for this error
    pub fn recovery_action(&self) -> RecoveryAction {
        match self {
            // Immediate retry
            Self::NetworkTimeout { .. } | Self::ConnectionLost { .. } => {
                RecoveryAction::RetryImmediate
            }

            // Retry with backoff
            Self::NetworkError { .. } | Self::DatabaseLocked { .. } => {
                RecoveryAction::RetryWithBackoff
            }

            // Database repair
            Self::DatabaseCorrupted { .. } => RecoveryAction::RepairDatabase,

            // Restore backup
            Self::MigrationFailed { .. } | Self::ConfigurationCorrupted { .. } => {
                RecoveryAction::RestoreBackup
            }

            // Disable feature
            Self::UnsupportedFormat { .. }
            | Self::ContentSourceUnavailable { .. }
            | Self::PlaybackDeviceError { .. }
            | Self::CacheCorrupted { .. } => RecoveryAction::DisableFeature,

            // Safe shutdown
            Self::OutOfMemory { .. } | Self::TooManyOpenFiles { .. } => {
                RecoveryAction::SafeShutdown
            }

            // User intervention required
            Self::DiskFull { .. } | Self::PermissionDenied { .. } | Self::SyncAuthFailed { .. } => {
                RecoveryAction::UserIntervention
            }

            // Default to user intervention for safety
            _ => RecoveryAction::UserIntervention,
        }
    }

    /// Returns a user-friendly error message suitable for display in the UI
    pub fn user_message(&self) -> String {
        match self {
            Self::NetworkError { .. } | Self::NetworkTimeout { .. } => {
                "Cannot connect to the internet. Please check your connection.".to_string()
            }
            Self::ConnectionLost { .. } => {
                "Connection was interrupted. Tap to retry.".to_string()
            }
            Self::InvalidUrl { .. } => "The link provided is not valid.".to_string(),

            Self::DatabaseError { .. } | Self::DatabaseLocked { .. } => {
                "Database is temporarily unavailable. Please try again.".to_string()
            }
            Self::DatabaseCorrupted { .. } => {
                "The app's database is damaged and needs repair. Your data will be restored from the latest backup.".to_string()
            }
            Self::MigrationFailed { .. } => {
                "Failed to update the app's database. Restoring from backup...".to_string()
            }
            Self::RecordNotFound { .. } => "The requested item was not found.".to_string(),

            Self::UnsupportedFormat { format, .. } => {
                format!("This audio format ({}) is not supported.", format)
            }
            Self::AudioDecodeError { .. } => {
                "Cannot play this audio file. It may be corrupted or in an unsupported format."
                    .to_string()
            }
            Self::CorruptedAudioFile { .. } => {
                "This audio file is damaged and cannot be played.".to_string()
            }
            Self::PlaybackDeviceError { .. } => {
                "Cannot access audio playback. Please check your device settings.".to_string()
            }
            Self::InvalidPosition { .. } => {
                "Cannot seek to that position in the audio.".to_string()
            }

            Self::FileNotFound { .. } => "The file was not found. It may have been moved or deleted.".to_string(),
            Self::PermissionDenied { .. } => {
                "Permission denied. Please grant storage access in Settings.".to_string()
            }
            Self::DiskFull { .. } => {
                "Not enough storage space. Please free up some space and try again.".to_string()
            }
            Self::IoError { .. } => "A file operation failed. Please try again.".to_string(),

            Self::MetadataParseError { .. } | Self::InvalidMetadata { .. } => {
                "Cannot read this file's information.".to_string()
            }
            Self::MissingMetadata { .. } => {
                "This file is missing important information.".to_string()
            }

            Self::ContentSourceUnavailable { provider, .. } => {
                format!("{} is currently unavailable. Please try again later.", provider)
            }
            Self::ContentNotFound { .. } => "The requested content was not found.".to_string(),
            Self::InvalidContentResponse { .. } => {
                "Received invalid data from the server.".to_string()
            }

            Self::InvalidConfiguration { setting, .. } => {
                format!("Invalid setting: {}. Please check your configuration.", setting)
            }
            Self::ConfigurationCorrupted { .. } => {
                "App settings are corrupted. Resetting to defaults...".to_string()
            }

            Self::SyncConflict { .. } => {
                "This item was modified on multiple devices. Please choose which version to keep."
                    .to_string()
            }
            Self::SyncAuthFailed { .. } => {
                "Sync authentication failed. Please sign in again.".to_string()
            }

            Self::CacheWriteFailed { .. } => {
                "Cannot save cached data. Streaming may be slower.".to_string()
            }
            Self::CacheCorrupted { .. } => {
                "Cache is corrupted and will be cleared.".to_string()
            }

            Self::OutOfMemory { .. } => {
                "Out of memory. The app will restart to free up resources.".to_string()
            }
            Self::TooManyOpenFiles { .. } => {
                "Too many files are open. The app will restart.".to_string()
            }

            Self::InternalError { .. } => {
                "An unexpected error occurred. Please try again.".to_string()
            }
            Self::Cancelled { .. } => "Operation was cancelled.".to_string(),
            Self::InvalidArgument { .. } => "Invalid input provided.".to_string(),
        }
    }

    /// Returns true if this error should be logged at ERROR level
    pub fn is_critical(&self) -> bool {
        self.severity() == ErrorSeverity::Fatal
    }

    /// Returns true if this error can be automatically retried
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.recovery_action(),
            RecoveryAction::RetryImmediate | RecoveryAction::RetryWithBackoff
        )
    }

    /// Helper to create a network error from any error type
    pub fn network<E: std::error::Error + Send + Sync + 'static>(
        message: impl Into<String>,
        source: E,
    ) -> Self {
        Self::NetworkError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Helper to create a database error from any error type
    pub fn database<E: std::error::Error + Send + Sync + 'static>(
        message: impl Into<String>,
        source: E,
    ) -> Self {
        Self::DatabaseError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Helper to create an audio decode error from any error type
    pub fn audio_decode<E: std::error::Error + Send + Sync + 'static>(
        message: impl Into<String>,
        source: E,
    ) -> Self {
        Self::AudioDecodeError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

/// Convenience type alias for Results using AppError
pub type Result<T> = std::result::Result<T, AppError>;

// Implement From for common error types
impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Self::FileNotFound {
                path: PathBuf::from("unknown"),
            },
            io::ErrorKind::PermissionDenied => Self::PermissionDenied {
                operation: "file operation".to_string(),
                path: PathBuf::from("unknown"),
            },
            _ => Self::IoError {
                message: err.to_string(),
                source: err,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_recovery_action_display() {
        assert_eq!(
            RecoveryAction::RetryImmediate.to_string(),
            "Retrying immediately"
        );
        assert_eq!(
            RecoveryAction::RetryWithBackoff.to_string(),
            "Retrying with backoff"
        );
        assert_eq!(
            RecoveryAction::DisableFeature.to_string(),
            "Disabling feature"
        );
        assert_eq!(
            RecoveryAction::RepairDatabase.to_string(),
            "Repairing database"
        );
        assert_eq!(
            RecoveryAction::RestoreBackup.to_string(),
            "Restoring from backup"
        );
        assert_eq!(
            RecoveryAction::SafeShutdown.to_string(),
            "Performing safe shutdown"
        );
        assert_eq!(
            RecoveryAction::UserIntervention.to_string(),
            "User intervention required"
        );
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Recoverable < ErrorSeverity::Degraded);
        assert!(ErrorSeverity::Degraded < ErrorSeverity::Fatal);
    }

    #[test]
    fn test_network_error_severity() {
        let err = AppError::NetworkError {
            message: "Connection failed".to_string(),
            source: None,
        };
        assert_eq!(err.severity(), ErrorSeverity::Recoverable);
        assert_eq!(err.recovery_action(), RecoveryAction::RetryWithBackoff);
        assert!(err.is_retryable());
        assert!(!err.is_critical());
    }

    #[test]
    fn test_network_timeout_severity() {
        let err = AppError::NetworkTimeout {
            operation: "download".to_string(),
            seconds: 30,
        };
        assert_eq!(err.severity(), ErrorSeverity::Recoverable);
        assert_eq!(err.recovery_action(), RecoveryAction::RetryImmediate);
        assert!(err.is_retryable());
    }

    #[test]
    fn test_database_corrupted_severity() {
        let err = AppError::DatabaseCorrupted {
            details: "Invalid header".to_string(),
        };
        assert_eq!(err.severity(), ErrorSeverity::Fatal);
        assert_eq!(err.recovery_action(), RecoveryAction::RepairDatabase);
        assert!(!err.is_retryable());
        assert!(err.is_critical());
    }

    #[test]
    fn test_unsupported_format_severity() {
        let err = AppError::UnsupportedFormat {
            format: "WMA".to_string(),
            file: PathBuf::from("/path/to/file.wma"),
        };
        assert_eq!(err.severity(), ErrorSeverity::Degraded);
        assert_eq!(err.recovery_action(), RecoveryAction::DisableFeature);
        assert!(!err.is_retryable());
        assert!(!err.is_critical());
    }

    #[test]
    fn test_out_of_memory_severity() {
        let err = AppError::OutOfMemory {
            requested_bytes: 1_000_000,
        };
        assert_eq!(err.severity(), ErrorSeverity::Fatal);
        assert_eq!(err.recovery_action(), RecoveryAction::SafeShutdown);
        assert!(err.is_critical());
    }

    #[test]
    fn test_disk_full_severity() {
        let err = AppError::DiskFull {
            needed_bytes: 1_000_000,
            available_bytes: 100_000,
        };
        assert_eq!(err.severity(), ErrorSeverity::Fatal);
        assert_eq!(err.recovery_action(), RecoveryAction::UserIntervention);
        assert!(err.is_critical());
    }

    #[test]
    fn test_user_messages_are_friendly() {
        let err = AppError::NetworkError {
            message: "TCP connection refused".to_string(),
            source: None,
        };
        let msg = err.user_message();
        assert!(!msg.contains("TCP"));
        assert!(msg.contains("internet"));

        let err2 = AppError::DatabaseCorrupted {
            details: "SQLite header corrupted".to_string(),
        };
        let msg2 = err2.user_message();
        assert!(!msg2.contains("SQLite"));
        assert!(msg2.contains("database"));
    }

    #[test]
    fn test_error_display() {
        let err = AppError::FileNotFound {
            path: PathBuf::from("/test/file.mp3"),
        };
        let display = format!("{}", err);
        assert!(display.contains("File not found"));
        assert!(display.contains("/test/file.mp3"));
    }

    #[test]
    fn test_network_helper() {
        let inner_err = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
        let err = AppError::network("Failed to connect", inner_err);

        assert!(matches!(err, AppError::NetworkError { .. }));
        if let AppError::NetworkError { message, source } = err {
            assert_eq!(message, "Failed to connect");
            assert!(source.is_some());
        }
    }

    #[test]
    fn test_database_helper() {
        let inner_err = io::Error::new(io::ErrorKind::Other, "Database locked");
        let err = AppError::database("Query failed", inner_err);

        assert!(matches!(err, AppError::DatabaseError { .. }));
    }

    #[test]
    fn test_audio_decode_helper() {
        let inner_err = io::Error::new(io::ErrorKind::InvalidData, "Invalid frame");
        let err = AppError::audio_decode("Decode failed", inner_err);

        assert!(matches!(err, AppError::AudioDecodeError { .. }));
    }

    #[test]
    fn test_from_io_error_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let app_err: AppError = io_err.into();

        assert!(matches!(app_err, AppError::FileNotFound { .. }));
    }

    #[test]
    fn test_from_io_error_permission_denied() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let app_err: AppError = io_err.into();

        assert!(matches!(app_err, AppError::PermissionDenied { .. }));
    }

    #[test]
    fn test_from_io_error_other() {
        let io_err = io::Error::new(io::ErrorKind::Other, "Unknown error");
        let app_err: AppError = io_err.into();

        assert!(matches!(app_err, AppError::IoError { .. }));
    }

    #[test]
    fn test_result_type_alias() {
        fn test_function() -> Result<i32> {
            Ok(42)
        }

        assert_eq!(test_function().unwrap(), 42);
    }

    #[test]
    fn test_all_network_errors_have_correct_severity() {
        let errors = vec![
            AppError::NetworkError {
                message: "test".to_string(),
                source: None,
            },
            AppError::NetworkTimeout {
                operation: "test".to_string(),
                seconds: 10,
            },
            AppError::ConnectionLost {
                message: "test".to_string(),
            },
        ];

        for err in errors {
            assert_eq!(err.severity(), ErrorSeverity::Recoverable);
            assert!(err.is_retryable());
        }
    }

    #[test]
    fn test_all_database_errors_have_correct_recovery() {
        let err1 = AppError::DatabaseLocked {
            operation: "write".to_string(),
        };
        assert_eq!(err1.recovery_action(), RecoveryAction::RetryWithBackoff);

        let err2 = AppError::DatabaseCorrupted {
            details: "test".to_string(),
        };
        assert_eq!(err2.recovery_action(), RecoveryAction::RepairDatabase);

        let err3 = AppError::MigrationFailed {
            version: "v2".to_string(),
            reason: "test".to_string(),
        };
        assert_eq!(err3.recovery_action(), RecoveryAction::RestoreBackup);
    }

    #[test]
    fn test_content_source_errors() {
        let err = AppError::ContentSourceUnavailable {
            provider: "LibriVox".to_string(),
            reason: "Server down".to_string(),
        };

        assert_eq!(err.severity(), ErrorSeverity::Degraded);
        assert_eq!(err.recovery_action(), RecoveryAction::DisableFeature);
        assert!(err.user_message().contains("LibriVox"));
    }

    #[test]
    fn test_sync_errors_phase3_prep() {
        let err1 = AppError::SyncConflict {
            entity: "bookmark".to_string(),
        };
        assert_eq!(err1.severity(), ErrorSeverity::Degraded);

        let err2 = AppError::SyncAuthFailed {
            provider: "Google Drive".to_string(),
        };
        assert_eq!(err2.recovery_action(), RecoveryAction::UserIntervention);
    }

    #[test]
    fn test_cache_errors() {
        let err1 = AppError::CacheWriteFailed {
            reason: "Disk full".to_string(),
        };
        assert_eq!(err1.severity(), ErrorSeverity::Recoverable);

        let err2 = AppError::CacheCorrupted {
            path: PathBuf::from("/cache"),
            reason: "Invalid header".to_string(),
        };
        assert_eq!(err2.severity(), ErrorSeverity::Degraded);
        assert_eq!(err2.recovery_action(), RecoveryAction::DisableFeature);
    }

    #[test]
    fn test_system_resource_errors() {
        let err1 = AppError::OutOfMemory {
            requested_bytes: 1_000_000,
        };
        assert_eq!(err1.recovery_action(), RecoveryAction::SafeShutdown);
        assert!(err1.is_critical());

        let err2 = AppError::TooManyOpenFiles { limit: 1024 };
        assert_eq!(err2.recovery_action(), RecoveryAction::SafeShutdown);
        assert!(err2.is_critical());
    }

    #[test]
    fn test_error_source_chain() {
        let inner = io::Error::new(io::ErrorKind::Other, "Inner error");
        let outer = AppError::network("Outer error", inner);

        // Verify that the source chain is preserved
        assert!(outer.source().is_some());
    }

    #[test]
    fn test_invalid_position_error() {
        let err = AppError::InvalidPosition {
            position: 5000,
            duration: 3000,
        };

        let display = format!("{}", err);
        assert!(display.contains("5000"));
        assert!(display.contains("3000"));
    }

    #[test]
    fn test_metadata_errors() {
        let err1 = AppError::MetadataParseError {
            file: PathBuf::from("/test.mp3"),
            reason: "Invalid ID3".to_string(),
        };
        assert!(!err1.is_critical());

        let err2 = AppError::InvalidMetadata {
            field: "duration".to_string(),
            value: "-5".to_string(),
        };
        let msg = err2.user_message();
        assert!(!msg.is_empty());

        let err3 = AppError::MissingMetadata {
            field: "title".to_string(),
            file: PathBuf::from("/test.mp3"),
        };
        assert_eq!(err3.severity(), ErrorSeverity::Degraded);
    }

    #[test]
    fn test_configuration_errors() {
        let err1 = AppError::InvalidConfiguration {
            setting: "playback_speed".to_string(),
            value: "5.0".to_string(),
            reason: "Must be between 0.5 and 3.0".to_string(),
        };
        assert!(!err1.is_retryable());

        let err2 = AppError::ConfigurationCorrupted {
            path: PathBuf::from("/config.toml"),
        };
        assert_eq!(err2.recovery_action(), RecoveryAction::RestoreBackup);
        assert!(err2.is_critical());
    }

    #[test]
    fn test_cancelled_error() {
        let err = AppError::Cancelled {
            operation: "download".to_string(),
        };
        assert!(!err.is_critical());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_invalid_argument_error() {
        let err = AppError::InvalidArgument {
            argument: "volume".to_string(),
            reason: "Must be between 0 and 100".to_string(),
        };
        assert!(!err.is_retryable());
    }
}
