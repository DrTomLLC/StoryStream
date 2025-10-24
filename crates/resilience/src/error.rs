// crates/resilience/src/error.rs
//! Error types for resilience operations

use thiserror::Error;

/// Result type for resilience operations
pub type ResilienceResult<T> = Result<T, ResilienceError>;

/// Errors that can occur in resilience operations
#[derive(Debug, Error)]
pub enum ResilienceError {
    /// Operation timed out
    #[error("Operation timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// All retry attempts exhausted
    #[error("All {attempts} retry attempts exhausted: {last_error}")]
    RetriesExhausted { attempts: usize, last_error: String },

    /// Circuit breaker is open
    #[error(
        "Circuit breaker is open (failures: {failures}, last failure: {last_failure_ago:?} ago)"
    )]
    CircuitBreakerOpen {
        failures: usize,
        last_failure_ago: std::time::Duration,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded (limit: {limit} per {window:?})")]
    RateLimitExceeded {
        limit: usize,
        window: std::time::Duration,
    },

    /// Operation was cancelled
    #[error("Operation was cancelled")]
    Cancelled,

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_error() {
        let err = ResilienceError::Timeout(std::time::Duration::from_secs(5));
        assert!(err.to_string().contains("timed out"));
        assert!(err.to_string().contains("5s"));
    }

    #[test]
    fn test_retries_exhausted_error() {
        let err = ResilienceError::RetriesExhausted {
            attempts: 3,
            last_error: "connection failed".to_string(),
        };
        assert!(err.to_string().contains("3"));
        assert!(err.to_string().contains("connection failed"));
    }

    #[test]
    fn test_circuit_breaker_error() {
        let err = ResilienceError::CircuitBreakerOpen {
            failures: 5,
            last_failure_ago: std::time::Duration::from_secs(10),
        };
        assert!(err.to_string().contains("Circuit breaker"));
        assert!(err.to_string().contains("5"));
    }

    #[test]
    fn test_rate_limit_error() {
        let err = ResilienceError::RateLimitExceeded {
            limit: 100,
            window: std::time::Duration::from_secs(60),
        };
        assert!(err.to_string().contains("Rate limit"));
        assert!(err.to_string().contains("100"));
    }
}
