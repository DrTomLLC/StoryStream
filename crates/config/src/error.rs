//! Error types for the configuration system

use std::path::PathBuf;
use thiserror::Error;

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Errors that can occur during configuration operations
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to read config file
    #[error("Failed to read config file at {path}: {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to write config file
    #[error("Failed to write config file at {path}: {source}")]
    WriteError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to parse config file
    #[error("Failed to parse config file at {path}: {source}")]
    ParseError {
        path: PathBuf,
        source: toml::de::Error,
    },

    /// Failed to serialize config
    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),

    /// Config file contains invalid values
    #[error("Config validation failed: {0}")]
    ValidationError(String),

    /// Failed to create config directory
    #[error("Failed to create config directory at {path}: {source}")]
    DirectoryCreationError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Config directory path could not be determined
    #[error("Could not determine config directory path: {reason}")]
    PathResolutionError { reason: String },

    /// Failed to create backup of old config
    #[error("Failed to backup config file: {source}")]
    BackupError { source: std::io::Error },

    /// Config file is locked by another process
    #[error("Config file is locked by another process")]
    FileLocked,

    /// Generic I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Validation error for a specific config field
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Path to the field (e.g., "player.default_volume")
    pub field: String,

    /// Human-readable error message
    pub message: String,

    /// The invalid value, if available
    pub value: Option<String>,
}

impl ValidationError {
    /// Creates a new validation error
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            value: None,
        }
    }

    /// Creates a validation error with the invalid value
    pub fn with_value(
        field: impl Into<String>,
        message: impl Into<String>,
        value: impl ToString,
    ) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            value: Some(value.to_string()),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field '{}': {}", self.field, self.message)?;
        if let Some(ref value) = self.value {
            write!(f, " (got: {})", value)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::new("player.volume", "must be between 0 and 100");
        assert_eq!(
            err.to_string(),
            "Field 'player.volume': must be between 0 and 100"
        );
    }

    #[test]
    fn test_validation_error_with_value() {
        let err =
            ValidationError::with_value("player.volume", "must be between 0 and 100", "150");
        assert_eq!(
            err.to_string(),
            "Field 'player.volume': must be between 0 and 100 (got: 150)"
        );
    }
}