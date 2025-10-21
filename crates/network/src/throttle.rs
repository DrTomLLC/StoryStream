// crates/network/src/throttle.rs
//! Bandwidth throttling for downloads

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug)]
struct TokenBucket {
    capacity: u64,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(bytes_per_second: u64) -> Self {
        Self {
            capacity: bytes_per_second,
            tokens: bytes_per_second as f64,
            refill_rate: bytes_per_second as f64,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        let new_tokens = elapsed * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.capacity as f64);
        self.last_refill = now;
    }

    fn consume(&mut self, bytes: usize) -> Result<(), Duration> {
        self.refill();

        if self.tokens >= bytes as f64 {
            self.tokens -= bytes as f64;
            Ok(())
        } else {
            let tokens_needed = bytes as f64 - self.tokens;
            let wait_secs = tokens_needed / self.refill_rate;
            Err(Duration::from_secs_f64(wait_secs))
        }
    }

    fn update_rate(&mut self, bytes_per_second: u64) {
        self.capacity = bytes_per_second;
        self.refill_rate = bytes_per_second as f64;
        self.tokens = self.tokens.min(self.capacity as f64);
    }
}

pub struct BandwidthThrottle {
    bucket: Arc<Mutex<TokenBucket>>,
}

impl BandwidthThrottle {
    pub fn new(bytes_per_second: u64) -> Self {
        Self {
            bucket: Arc::new(Mutex::new(TokenBucket::new(bytes_per_second))),
        }
    }

    pub async fn wait_for_capacity(&self, bytes: usize) {
        loop {
            let wait_duration = {
                let mut bucket = self.bucket.lock().await;
                match bucket.consume(bytes) {
                    Ok(()) => return,
                    Err(duration) => duration,
                }
            };

            tokio::time::sleep(wait_duration).await;
        }
    }

    pub async fn try_consume(&self, bytes: usize) -> bool {
        let mut bucket = self.bucket.lock().await;
        bucket.consume(bytes).is_ok()
    }

    pub async fn update_limit(&self, bytes_per_second: u64) {
        let mut bucket = self.bucket.lock().await;
        bucket.update_rate(bytes_per_second);
    }

    pub async fn get_limit(&self) -> u64 {
        let bucket = self.bucket.lock().await;
        bucket.capacity
    }

    pub async fn available_tokens(&self) -> u64 {
        let mut bucket = self.bucket.lock().await;
        bucket.refill();
        bucket.tokens as u64
    }
}

impl Clone for BandwidthThrottle {
    fn clone(&self) -> Self {
        Self {
            bucket: Arc::clone(&self.bucket),
        }
    }
}

pub struct AdaptiveThrottle {
    throttle: BandwidthThrottle,
    min_rate: u64,
    max_rate: u64,
    current_rate: Arc<Mutex<u64>>,
}

impl AdaptiveThrottle {
    pub fn new(min_rate: u64, max_rate: u64, initial_rate: u64) -> Self {
        Self {
            throttle: BandwidthThrottle::new(initial_rate),
            min_rate,
            max_rate,
            current_rate: Arc::new(Mutex::new(initial_rate)),
        }
    }

    pub async fn increase_bandwidth(&self) {
        let mut rate = self.current_rate.lock().await;
        let new_rate = (*rate as f64 * 1.1) as u64;
        *rate = new_rate.min(self.max_rate);
        self.throttle.update_limit(*rate).await;
    }

    pub async fn decrease_bandwidth(&self) {
        let mut rate = self.current_rate.lock().await;
        let new_rate = (*rate as f64 * 0.8) as u64;
        *rate = new_rate.max(self.min_rate);
        self.throttle.update_limit(*rate).await;
    }

    pub fn inner(&self) -> &BandwidthThrottle {
        &self.throttle
    }

    pub async fn current_rate(&self) -> u64 {
        *self.current_rate.lock().await
    }
}