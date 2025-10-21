// crates/network/src/lib.rs
//! Network utilities for HTTP requests and downloads

mod client;
mod connectivity;
mod download;
mod download_manager;
mod error;
mod progress;
mod resume;
mod throttle;

pub use client::{Client, ClientConfig};
pub use connectivity::ConnectivityChecker;
pub use download::DownloadManager;
pub use download_manager::{
    AdvancedDownloadManager, DownloadManagerConfig, DownloadStatus, DownloadTask, Priority,
    ProgressCallback,
};
pub use error::{NetworkError, NetworkResult};
pub use progress::{DownloadProgress, ProgressTracker};
pub use resume::{can_resume, ResumeInfo, ResumeManager};
pub use throttle::{AdaptiveThrottle, BandwidthThrottle};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_exports_accessible() {
        // Verify all types are exported
        let client = Client::new().expect("Failed to create client");
        let _: DownloadManager = DownloadManager::new(client.clone());
        let _: ConnectivityChecker = ConnectivityChecker::new(client);
        let _: ProgressTracker = ProgressTracker::new(Some(1000));
    }
}