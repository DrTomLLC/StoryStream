// crates/resilience/src/lib.rs
//! Resilience patterns for fault-tolerant operations
//!
//! This module provides resilience patterns including:
//! - Retry with exponential backoff
//! - Circuit breaker
//! - Timeout handling
//! - Rate limiting
//!
//! # Example
//!
//! ```rust
//! use storystream_resilience::{RetryPolicy, CircuitBreaker, CircuitBreakerConfig};
//! use std::time::Duration;
//!
//! // Retry with exponential backoff
//! let policy = RetryPolicy::new(3)
//!     .with_initial_delay(Duration::from_millis(100));
//!
//! // Circuit breaker
//! let cb_config = CircuitBreakerConfig::new(5, Duration::from_secs(60));
//! let cb = CircuitBreaker::new(cb_config);
//! ```

mod circuit_breaker;
mod error;
mod rate_limiter;
mod retry;
mod timeout;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use error::{ResilienceError, ResilienceResult};
pub use rate_limiter::RateLimiter;
pub use retry::{with_retry, RetryPolicy};
pub use timeout::{with_timeout, Timeout};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_exports_accessible() {
        // Verify all types are exported
        let _: RetryPolicy = RetryPolicy::default();
        let _: CircuitBreakerConfig = CircuitBreakerConfig::default();
        let _: CircuitBreaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let _: RateLimiter = RateLimiter::new(100, std::time::Duration::from_secs(1));
        let _: Timeout = Timeout::new(std::time::Duration::from_secs(5));
    }
}
