// crates/tui/src/error.rs
//! Error types for TUI

use thiserror::Error;

/// Result type for TUI operations
pub type TuiResult<T> = Result<T, TuiError>;

/// Errors that can occur in the TUI
#[derive(Debug, Error)]
pub enum TuiError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal error
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Application error
    #[error("Application error: {0}")]
    Application(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Media engine error
    #[error("Media engine error: {0}")]
    MediaEngine(String),

    /// Initialization error (for integrated mode)
    #[error("Initialization error: {0}")]
    Initialization(String),

    /// Playback error (for integrated mode)
    #[error("Playback error: {0}")]
    PlaybackError(String),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TuiError::Terminal("screen error".to_string());
        assert!(err.to_string().contains("Terminal error"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let tui_err: TuiError = io_err.into();
        assert!(matches!(tui_err, TuiError::Io(_)));
    }

    #[test]
    fn test_initialization_error() {
        let err = TuiError::Initialization("Failed to setup".to_string());
        assert!(err.to_string().contains("Initialization error"));
    }

    #[test]
    fn test_playback_error() {
        let err = TuiError::PlaybackError("Audio failed".to_string());
        assert!(err.to_string().contains("Playback error"));
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            TuiError::Terminal("test".into()),
            TuiError::Application("test".into()),
            TuiError::Database("test".into()),
            TuiError::MediaEngine("test".into()),
            TuiError::Initialization("test".into()),
            TuiError::PlaybackError("test".into()),
            TuiError::Custom("test".into()),
        ];

        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }
}