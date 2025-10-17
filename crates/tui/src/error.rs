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
}