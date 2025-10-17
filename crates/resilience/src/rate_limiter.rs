// crates/resilience/src/rate_limiter.rs
//! Rate limiting implementation

use crate::error::{ResilienceError, ResilienceResult};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Token bucket rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    max_tokens: usize,
    refill_rate: Duration,
    state: Arc<Mutex<RateLimiterState>>,
}

#[derive(Debug)]
struct RateLimiterState {
    tokens: usize,
    last_refill: Instant,
    requests: VecDeque<Instant>,
}

impl RateLimiter {
    /// Creates a new rate limiter
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_tokens: max_requests,
            refill_rate: window,
            state: Arc::new(Mutex::new(RateLimiterState {
                tokens: max_requests,
                last_refill: Instant::now(),
                requests: VecDeque::new(),
            })),
        }
    }

    /// Attempts to acquire a token
    pub fn try_acquire(&self) -> ResilienceResult<()> {
        let mut state = self.state.lock()
            .map_err(|_| ResilienceError::Custom("Lock poisoned".to_string()))?;

        self.refill_tokens(&mut state);

        // Clean old requests
        let cutoff = Instant::now() - self.refill_rate;
        while let Some(&oldest) = state.requests.front() {
            if oldest < cutoff {
                state.requests.pop_front();
            } else {
                break;
            }
        }

        if state.requests.len() < self.max_tokens {
            state.requests.push_back(Instant::now());
            Ok(())
        } else {
            Err(ResilienceError::RateLimitExceeded {
                limit: self.max_tokens,
                window: self.refill_rate,
            })
        }
    }

    /// Refills tokens based on elapsed time
    fn refill_tokens(&self, state: &mut RateLimiterState) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill);

        if elapsed >= self.refill_rate {
            state.tokens = self.max_tokens;
            state.last_refill = now;
        }
    }

    /// Gets the maximum number of requests allowed
    pub fn max_requests(&self) -> usize {
        self.max_tokens
    }

    /// Gets the time window
    pub fn window(&self) -> Duration {
        self.refill_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1));

        for _ in 0..5 {
            assert!(limiter.try_acquire().is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(3, Duration::from_secs(1));

        // Use up all tokens
        for _ in 0..3 {
            assert!(limiter.try_acquire().is_ok());
        }

        // Next request should be rate limited
        let result = limiter.try_acquire();
        assert!(result.is_err());
        assert!(matches!(result, Err(ResilienceError::RateLimitExceeded { .. })));
    }

    #[test]
    fn test_rate_limiter_refills_after_window() {
        let limiter = RateLimiter::new(2, Duration::from_millis(50));

        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_err());

        // Wait for refill
        std::thread::sleep(Duration::from_millis(60));

        // Should be able to acquire again
        assert!(limiter.try_acquire().is_ok());
    }

    #[test]
    fn test_rate_limiter_config() {
        let limiter = RateLimiter::new(100, Duration::from_secs(60));
        assert_eq!(limiter.max_requests(), 100);
        assert_eq!(limiter.window(), Duration::from_secs(60));
    }
}