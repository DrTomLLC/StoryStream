// crates/resilience/examples/resilience_demo.rs
//! Demonstration of resilience patterns

use storystream_resilience::{
    with_retry, CircuitBreaker, CircuitBreakerConfig, RateLimiter, RetryPolicy, Timeout,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    println!("Resilience Patterns Demo");
    println!("========================\n");

    demo_retry();
    println!();
    demo_circuit_breaker();
    println!();
    demo_rate_limiter();
    println!();
    demo_timeout();
}

fn demo_retry() {
    println!("1. Retry Pattern");
    println!("----------------");

    let policy = RetryPolicy::new(3)
        .with_initial_delay(Duration::from_millis(100))
        .with_multiplier(2.0);

    let attempt = Arc::new(Mutex::new(0));
    let attempt_clone = attempt.clone();

    let result = with_retry(&policy, || {
        let mut count = attempt_clone.lock().map_err(|_| "Lock failed")?;
        *count += 1;
        println!("  Attempt {}", *count);

        if *count < 3 {
            Err("Simulated failure")
        } else {
            Ok(42)
        }
    });

    match result {
        Ok(value) => println!("✓ Success after retries: {}", value),
        Err(e) => println!("✗ Failed: {}", e),
    }
}

fn demo_circuit_breaker() {
    println!("2. Circuit Breaker Pattern");
    println!("--------------------------");

    let config = CircuitBreakerConfig::new(3, Duration::from_millis(100));
    let cb = CircuitBreaker::new(config);

    // Cause failures to open circuit
    for i in 1..=5 {
        let result = cb.call(|| {
            if i <= 3 {
                Err::<i32, _>("Service unavailable")
            } else {
                Ok(42)
            }
        });

        match result {
            Ok(_) => println!("  Request {}: ✓ Success", i),
            Err(e) => println!("  Request {}: ✗ {}", i, e),
        }
    }

    println!("  Circuit state: {:?}", cb.state());

    // Wait for timeout
    std::thread::sleep(Duration::from_millis(150));
    println!("  Waited for timeout...");

    let result = cb.call(|| Ok::<_, String>(42));
    match result {
        Ok(_) => println!("  After timeout: ✓ Request succeeded"),
        Err(e) => println!("  After timeout: ✗ {}", e),
    }

    println!("  Circuit state: {:?}", cb.state());
}

fn demo_rate_limiter() {
    println!("3. Rate Limiter Pattern");
    println!("-----------------------");

    let limiter = RateLimiter::new(5, Duration::from_secs(1));

    println!("  Limit: {} requests per second", limiter.max_requests());

    for i in 1..=7 {
        match limiter.try_acquire() {
            Ok(()) => println!("  Request {}: ✓ Allowed", i),
            Err(e) => println!("  Request {}: ✗ {}", i, e),
        }
    }
}

fn demo_timeout() {
    println!("4. Timeout Pattern");
    println!("------------------");

    let timeout = Timeout::new(Duration::from_millis(50));

    // Fast operation
    let result = timeout.execute(|| {
        std::thread::sleep(Duration::from_millis(10));
        42
    });

    match result {
        Ok(value) => println!("  Fast operation: ✓ Completed: {}", value),
        Err(e) => println!("  Fast operation: ✗ {}", e),
    }

    // Slow operation
    let result = timeout.execute(|| {
        std::thread::sleep(Duration::from_millis(100));
        42
    });

    match result {
        Ok(value) => println!("  Slow operation: ✓ Completed: {}", value),
        Err(e) => println!("  Slow operation: ✗ {}", e),
    }
}