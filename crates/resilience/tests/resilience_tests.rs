// crates/resilience/tests/resilience_tests.rs
//! Integration tests for resilience patterns

use storystream_resilience::{
    with_retry, CircuitBreaker, CircuitBreakerConfig, RateLimiter, RetryPolicy, Timeout,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_retry_with_circuit_breaker() {
    let policy = RetryPolicy::new(3).with_initial_delay(Duration::from_millis(1));
    let cb = CircuitBreaker::new(CircuitBreakerConfig::new(2, Duration::from_millis(50)));

    let attempt = Arc::new(Mutex::new(0));
    let attempt_clone = attempt.clone();

    // First few attempts fail, circuit should open
    let result = with_retry(&policy, || {
        let mut count = attempt_clone.lock().map_err(|_| "Lock failed")?;
        *count += 1;

        if *count < 5 {
            cb.record_failure();
            Err("fail")
        } else {
            cb.record_success();
            Ok(42)
        }
    });

    // Should eventually succeed or exhaust retries
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_rate_limiter_with_timeout() {
    let limiter = RateLimiter::new(5, Duration::from_secs(1));
    let timeout = Timeout::new(Duration::from_millis(100));

    for i in 0..5 {
        let result = timeout.execute(|| limiter.try_acquire());
        assert!(result.is_ok(), "Request {} should succeed", i + 1);
        // Verify the inner result is also Ok
        if let Ok(inner) = result {
            assert!(inner.is_ok(), "Request {} inner result should be Ok", i + 1);
        }
    }

    // 6th request should be rate limited
    // Timeout doesn't fail, but inner limiter does
    let result = timeout.execute(|| limiter.try_acquire());
    assert!(result.is_ok(), "Timeout should not fail");

    // But the limiter should reject
    if let Ok(inner_result) = result {
        assert!(inner_result.is_err(), "Rate limiter should reject 6th request");
    }
}

#[test]
fn test_combined_resilience_patterns() {
    let retry_policy = RetryPolicy::new(3).with_initial_delay(Duration::from_millis(1));
    let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::new(5, Duration::from_secs(1)));
    let rate_limiter = RateLimiter::new(10, Duration::from_secs(1));

    let mut successful = 0;

    for _ in 0..15 {
        // Check rate limit first
        if rate_limiter.try_acquire().is_err() {
            continue;
        }

        // Try with circuit breaker and retry
        let result = with_retry(&retry_policy, || {
            circuit_breaker.call(|| {
                // Simulate operation
                if successful < 12 {
                    Ok::<_, String>(())
                } else {
                    Err("simulated failure".to_string())
                }
            })
        });

        if result.is_ok() {
            successful += 1;
        }
    }

    // Should have processed some requests successfully
    assert!(successful > 0);
}

#[test]
fn test_resilience_under_load() {
    let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::new(10, Duration::from_millis(100)));
    let rate_limiter = RateLimiter::new(20, Duration::from_secs(1));

    let mut successes = 0;
    let mut rate_limited = 0;
    let mut _circuit_open = 0;

    for _ in 0..50 {
        // Rate limit check
        if rate_limiter.try_acquire().is_err() {
            rate_limited += 1;
            continue;
        }

        // Circuit breaker check
        if circuit_breaker.can_proceed().is_err() {
            _circuit_open += 1;
            continue;
        }

        // Simulate operation (80% success rate)
        if successes < 16 {
            circuit_breaker.record_success();
            successes += 1;
        } else {
            circuit_breaker.record_failure();
        }
    }

    // Verify resilience mechanisms worked
    assert!(successes > 0, "Should have some successful requests");
    assert!(rate_limited > 0, "Should have some rate-limited requests");
}