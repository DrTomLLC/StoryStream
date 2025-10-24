// crates/network/src/progress.rs
//! Download progress tracking

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Download progress information
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Total bytes to download (if known)
    pub total_bytes: Option<u64>,
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Download speed in bytes per second
    pub bytes_per_second: f64,
    /// Estimated time remaining
    pub estimated_remaining: Option<Duration>,
    /// Start time
    start_time: Instant,
}

impl DownloadProgress {
    /// Creates a new progress tracker
    pub fn new(total_bytes: Option<u64>) -> Self {
        Self {
            total_bytes,
            downloaded_bytes: 0,
            bytes_per_second: 0.0,
            estimated_remaining: None,
            start_time: Instant::now(),
        }
    }

    /// Updates progress with new downloaded bytes
    pub fn update(&mut self, additional_bytes: u64) {
        self.downloaded_bytes += additional_bytes;

        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.bytes_per_second = self.downloaded_bytes as f64 / elapsed;
        }

        // Calculate estimated remaining time
        if let Some(total) = self.total_bytes {
            if self.bytes_per_second > 0.0 {
                let remaining_bytes = total.saturating_sub(self.downloaded_bytes);
                let seconds_remaining = remaining_bytes as f64 / self.bytes_per_second;
                self.estimated_remaining = Some(Duration::from_secs_f64(seconds_remaining));
            }
        }
    }

    /// Returns progress as a percentage (0-100)
    pub fn percentage(&self) -> Option<f64> {
        self.total_bytes.map(|total| {
            if total == 0 {
                100.0
            } else {
                (self.downloaded_bytes as f64 / total as f64 * 100.0).min(100.0)
            }
        })
    }

    /// Returns true if download is complete
    pub fn is_complete(&self) -> bool {
        if let Some(total) = self.total_bytes {
            self.downloaded_bytes >= total
        } else {
            false
        }
    }

    /// Returns download speed in MB/s
    pub fn speed_mbps(&self) -> f64 {
        self.bytes_per_second / 1_000_000.0
    }
}

/// Thread-safe progress tracker
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    inner: Arc<Mutex<DownloadProgress>>,
}

impl ProgressTracker {
    /// Creates a new progress tracker
    pub fn new(total_bytes: Option<u64>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DownloadProgress::new(total_bytes))),
        }
    }

    /// Updates progress
    pub fn update(&self, bytes: u64) {
        if let Ok(mut progress) = self.inner.lock() {
            progress.update(bytes);
        }
    }

    /// Gets current progress
    pub fn get(&self) -> Option<DownloadProgress> {
        self.inner.lock().ok().map(|p| p.clone())
    }

    /// Gets progress percentage
    pub fn percentage(&self) -> Option<f64> {
        self.inner.lock().ok().and_then(|p| p.percentage())
    }

    /// Checks if download is complete
    pub fn is_complete(&self) -> bool {
        self.inner
            .lock()
            .ok()
            .map(|p| p.is_complete())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_new() {
        let progress = DownloadProgress::new(Some(1000));
        assert_eq!(progress.total_bytes, Some(1000));
        assert_eq!(progress.downloaded_bytes, 0);
        assert!(!progress.is_complete());
    }

    #[test]
    fn test_progress_update() {
        let mut progress = DownloadProgress::new(Some(1000));
        progress.update(250);
        assert_eq!(progress.downloaded_bytes, 250);
        assert_eq!(progress.percentage(), Some(25.0));
    }

    #[test]
    fn test_progress_percentage() {
        let mut progress = DownloadProgress::new(Some(1000));
        assert_eq!(progress.percentage(), Some(0.0));

        progress.update(500);
        assert_eq!(progress.percentage(), Some(50.0));

        progress.update(500);
        assert_eq!(progress.percentage(), Some(100.0));
    }

    #[test]
    fn test_progress_complete() {
        let mut progress = DownloadProgress::new(Some(1000));
        assert!(!progress.is_complete());

        progress.update(1000);
        assert!(progress.is_complete());
    }

    #[test]
    fn test_progress_unknown_size() {
        let progress = DownloadProgress::new(None);
        assert_eq!(progress.percentage(), None);
        assert!(!progress.is_complete());
    }

    #[test]
    fn test_progress_tracker() {
        let tracker = ProgressTracker::new(Some(1000));

        tracker.update(500);
        assert_eq!(tracker.percentage(), Some(50.0));

        tracker.update(500);
        assert!(tracker.is_complete());
    }

    #[test]
    fn test_speed_calculation() {
        let mut progress = DownloadProgress::new(Some(1000));
        std::thread::sleep(Duration::from_millis(10));
        progress.update(1000);

        assert!(progress.bytes_per_second > 0.0);
        assert!(progress.speed_mbps() > 0.0);
    }

    #[test]
    fn test_progress_zero_total() {
        let progress = DownloadProgress::new(Some(0));
        assert_eq!(progress.percentage(), Some(100.0));
    }
}
