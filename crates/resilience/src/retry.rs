// crates/resilience/src/retry.rs
//! Retry policies with exponential backoff

use crate::error::{ResilienceError};
use std::time::Duration;

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including the first attempt)
    max_attempts: usize,
    /// Initial delay between retries
    initial_delay: Duration,
    /// Maximum delay between retries
    max_delay: Duration,
    /// Backoff multiplier
    multiplier: f64,
    /// Whether to use jitter
    use_jitter: bool,
}

impl RetryPolicy {
    /// Creates a new retry policy
    pub fn new(max_attempts: usize) -> Self {
        Self {
            max_attempts,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            use_jitter: true,
        }
    }

    /// Sets the initial delay
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Sets the maximum delay
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Sets the backoff multiplier
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Sets whether to use jitter
    pub fn with_jitter(mut self, use_jitter: bool) -> Self {
        self.use_jitter = use_jitter;
        self
    }

    /// Calculates the delay for a given attempt
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let base_delay = self.initial_delay.as_millis() as f64
            * self.multiplier.powi((attempt - 1) as i32);

        let capped_delay = base_delay.min(self.max_delay.as_millis() as f64);

        let final_delay = if self.use_jitter {
            // Add up to 25% jitter
            let jitter_factor = 0.75 + (attempt as f64 * 0.1 % 0.25);
            capped_delay * jitter_factor
        } else {
            capped_delay
        };

        Duration::from_millis(final_delay as u64)
    }

    /// Returns the maximum number of attempts
    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    /// Checks if an error is retryable (default: all errors are retryable)
    pub fn is_retryable<E>(&self, _error: &E) -> bool {
        true
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Executes an operation with retry logic
pub fn with_retry<F, T, E>(policy: &RetryPolicy, mut operation: F) -> Result<T, ResilienceError>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut last_error = String::new();

    while attempt < policy.max_attempts() {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = e.to_string();
                attempt += 1;

                if attempt >= policy.max_attempts() {
                    break;
                }

                // Simulate delay (in real async code, use tokio::time::sleep)
                let delay = policy.delay_for_attempt(attempt);
                std::thread::sleep(delay);
            }
        }
    }

    Err(ResilienceError::RetriesExhausted {
        attempts: attempt,
        last_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts(), 3);
    }

    #[test]
    fn test_retry_policy_builder() {
        let policy = RetryPolicy::new(5)
            .with_initial_delay(Duration::from_millis(200))
            .with_max_delay(Duration::from_secs(60))
            .with_multiplier(3.0)
            .with_jitter(false);

        assert_eq!(policy.max_attempts(), 5);
        assert_eq!(policy.initial_delay, Duration::from_millis(200));
        assert_eq!(policy.max_delay, Duration::from_secs(60));
        assert_eq!(policy.multiplier, 3.0);
        assert!(!policy.use_jitter);
    }

    #[test]
    fn test_exponential_backoff() {
        let policy = RetryPolicy::new(4)
            .with_initial_delay(Duration::from_millis(100))
            .with_multiplier(2.0)
            .with_jitter(false);

        assert_eq!(policy.delay_for_attempt(0), Duration::from_millis(0));
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(200));
        assert_eq!(policy.delay_for_attempt(3), Duration::from_millis(400));
    }

    #[test]
    fn test_max_delay_capping() {
        let policy = RetryPolicy::new(10)
            .with_initial_delay(Duration::from_secs(1))
            .with_max_delay(Duration::from_secs(5))
            .with_multiplier(2.0)
            .with_jitter(false);

        // Should cap at max_delay
        let delay = policy.delay_for_attempt(10);
        assert!(delay <= Duration::from_secs(5));
    }

    #[test]
    fn test_with_retry_success_first_attempt() {
        let policy = RetryPolicy::new(3);
        let mut call_count = 0;

        let result = with_retry(&policy, || {
            call_count += 1;
            Ok::<_, String>(42)
        });

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
        assert_eq!(call_count, 1);
    }

    #[test]
    fn test_with_retry_success_after_failures() {
        let policy = RetryPolicy::new(3)
            .with_initial_delay(Duration::from_millis(1));
        let mut call_count = 0;

        let result = with_retry(&policy, || {
            call_count += 1;
            if call_count < 3 {
                Err("temporary error")
            } else {
                Ok(42)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
        assert_eq!(call_count, 3);
    }

    #[test]
    fn test_with_retry_all_attempts_fail() {
        let policy = RetryPolicy::new(3)
            .with_initial_delay(Duration::from_millis(1));
        let mut call_count = 0;

        let result = with_retry(&policy, || {
            call_count += 1;
            Err::<i32, _>("persistent error")
        });

        assert!(result.is_err());
        assert_eq!(call_count, 3);

        if let Err(ResilienceError::RetriesExhausted { attempts, last_error }) = result {
            assert_eq!(attempts, 3);
            assert_eq!(last_error, "persistent error");
        } else {
            panic!("Expected RetriesExhausted error");
        }
    }
}