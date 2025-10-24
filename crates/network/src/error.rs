// crates/network/src/error.rs
//! Error types for network operations

use thiserror::Error;

/// Result type for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Errors that can occur during network operations
#[derive(Debug, Error)]
pub enum NetworkError {
    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Download failed
    #[error("Download failed: {0}")]
    DownloadFailed(String),

    /// Network unavailable
    #[error("Network is unavailable")]
    NetworkUnavailable,

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// Resilience error
    #[error("Resilience error: {0}")]
    Resilience(#[from] storystream_resilience::ResilienceError),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

impl NetworkError {
    /// Returns true if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            NetworkError::Timeout | NetworkError::NetworkUnavailable | NetworkError::Http(_)
        )
    }

    /// Returns true if the error is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        if let NetworkError::Http(e) = self {
            if let Some(status) = e.status() {
                return status.is_client_error();
            }
        }
        false
    }

    /// Returns true if the error is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        if let NetworkError::Http(e) = self {
            if let Some(status) = e.status() {
                return status.is_server_error();
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NetworkError::InvalidUrl("test".to_string());
        assert!(err.to_string().contains("Invalid URL"));
    }

    #[test]
    fn test_download_failed_error() {
        let err = NetworkError::DownloadFailed("connection reset".to_string());
        assert!(err.to_string().contains("Download failed"));
    }

    #[test]
    fn test_network_unavailable() {
        let err = NetworkError::NetworkUnavailable;
        assert!(err.to_string().contains("unavailable"));
    }

    #[test]
    fn test_retryable_errors() {
        assert!(NetworkError::Timeout.is_retryable());
        assert!(NetworkError::NetworkUnavailable.is_retryable());
        assert!(!NetworkError::InvalidUrl("test".to_string()).is_retryable());
    }
}
