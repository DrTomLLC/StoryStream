// crates/resilience/src/circuit_breaker.rs
//! Circuit breaker pattern implementation

use crate::error::{ResilienceError, ResilienceResult};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    failure_threshold: usize,
    /// Duration to wait before trying again after opening
    timeout: Duration,
    /// Number of successful requests needed to close from half-open
    success_threshold: usize,
}

impl CircuitBreakerConfig {
    /// Creates a new configuration
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            timeout,
            success_threshold: 2,
        }
    }

    /// Sets the success threshold
    pub fn with_success_threshold(mut self, threshold: usize) -> Self {
        self.success_threshold = threshold;
        self
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(60))
    }
}

/// Circuit breaker state
#[derive(Debug)]
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: usize,
    success_count: usize,
    last_failure_time: Option<Instant>,
}

/// Circuit breaker implementation
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitBreakerState>>,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
            })),
        }
    }

    /// Gets the current state
    pub fn state(&self) -> CircuitState {
        self.state.lock().map(|s| s.state).unwrap_or(CircuitState::Open)
    }

    /// Records a successful operation
    pub fn record_success(&self) {
        if let Ok(mut state) = self.state.lock() {
            match state.state {
                CircuitState::HalfOpen => {
                    state.success_count += 1;
                    if state.success_count >= self.config.success_threshold {
                        state.state = CircuitState::Closed;
                        state.failure_count = 0;
                        state.success_count = 0;
                    }
                }
                CircuitState::Closed => {
                    state.failure_count = 0;
                }
                CircuitState::Open => {}
            }
        }
    }

    /// Records a failed operation
    pub fn record_failure(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.failure_count += 1;
            state.last_failure_time = Some(Instant::now());
            state.success_count = 0;

            if state.failure_count >= self.config.failure_threshold {
                state.state = CircuitState::Open;
            }
        }
    }

    /// Checks if a request can proceed
    pub fn can_proceed(&self) -> ResilienceResult<()> {
        let mut state = self.state.lock()
            .map_err(|_| ResilienceError::Custom("Lock poisoned".to_string()))?;

        match state.state {
            CircuitState::Closed => Ok(()),
            CircuitState::HalfOpen => Ok(()),
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed() >= self.config.timeout {
                        state.state = CircuitState::HalfOpen;
                        state.success_count = 0;
                        Ok(())
                    } else {
                        Err(ResilienceError::CircuitBreakerOpen {
                            failures: state.failure_count,
                            last_failure_ago: last_failure.elapsed(),
                        })
                    }
                } else {
                    Err(ResilienceError::CircuitBreakerOpen {
                        failures: state.failure_count,
                        last_failure_ago: Duration::from_secs(0),
                    })
                }
            }
        }
    }

    /// Executes an operation through the circuit breaker
    pub fn call<F, T, E>(&self, operation: F) -> ResilienceResult<T>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        self.can_proceed()?;

        match operation() {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(ResilienceError::Custom(e.to_string()))
            }
        }
    }

    /// Resets the circuit breaker to closed state
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.state = CircuitState::Closed;
            state.failure_count = 0;
            state.success_count = 0;
            state.last_failure_time = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_opens_after_threshold() {
        let config = CircuitBreakerConfig::new(3, Duration::from_secs(1));
        let cb = CircuitBreaker::new(config);

        assert_eq!(cb.state(), CircuitState::Closed);

        // Record failures
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_rejects_when_open() {
        let config = CircuitBreakerConfig::new(2, Duration::from_secs(10));
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        let result = cb.can_proceed();
        assert!(result.is_err());
    }

    #[test]
    fn test_circuit_half_open_after_timeout() {
        let config = CircuitBreakerConfig::new(2, Duration::from_millis(50));
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        std::thread::sleep(Duration::from_millis(60));

        let result = cb.can_proceed();
        assert!(result.is_ok());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_closes_after_success_threshold() {
        let config = CircuitBreakerConfig::new(2, Duration::from_millis(50))
            .with_success_threshold(2);
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(60));
        let _ = cb.can_proceed();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Record successes
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_call_success() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());

        let result = cb.call(|| Ok::<_, String>(42));
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
    }

    #[test]
    fn test_circuit_breaker_call_failure() {
        let config = CircuitBreakerConfig::new(2, Duration::from_secs(1));
        let cb = CircuitBreaker::new(config);

        let _ = cb.call(|| Err::<i32, _>("error 1"));
        let _ = cb.call(|| Err::<i32, _>("error 2"));

        // Circuit should now be open
        assert_eq!(cb.state(), CircuitState::Open);

        // Next call should be rejected
        let result = cb.call(|| Ok::<_, String>(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let config = CircuitBreakerConfig::new(2, Duration::from_secs(1));
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}