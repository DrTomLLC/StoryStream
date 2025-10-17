// crates/resilience/src/timeout.rs
//! Timeout handling utilities

use crate::error::{ResilienceError, ResilienceResult};
use std::time::{Duration, Instant};

/// Executes an operation with a timeout
pub fn with_timeout<F, T>(duration: Duration, operation: F) -> ResilienceResult<T>
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = operation();

    if start.elapsed() > duration {
        Err(ResilienceError::Timeout(duration))
    } else {
        Ok(result)
    }
}

/// Timeout wrapper for operations
#[derive(Debug, Clone)]
pub struct Timeout {
    duration: Duration,
}

impl Timeout {
    /// Creates a new timeout
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }

    /// Gets the timeout duration
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Executes an operation with this timeout
    pub fn execute<F, T>(&self, operation: F) -> ResilienceResult<T>
    where
        F: FnOnce() -> T,
    {
        with_timeout(self.duration, operation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_success() {
        let result = with_timeout(Duration::from_secs(1), || {
            std::thread::sleep(Duration::from_millis(10));
            42
        });

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
    }

    #[test]
    fn test_timeout_exceeded() {
        let result = with_timeout(Duration::from_millis(10), || {
            std::thread::sleep(Duration::from_millis(50));
            42
        });

        assert!(result.is_err());
        assert!(matches!(result, Err(ResilienceError::Timeout(_))));
    }

    #[test]
    fn test_timeout_wrapper() {
        let timeout = Timeout::new(Duration::from_millis(100));

        let result = timeout.execute(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        });

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
    }

    #[test]
    fn test_timeout_duration() {
        let timeout = Timeout::new(Duration::from_secs(5));
        assert_eq!(timeout.duration(), Duration::from_secs(5));
    }
}