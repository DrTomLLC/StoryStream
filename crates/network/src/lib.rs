// crates/network/src/lib.rs
//! Network utilities for HTTP requests and downloads
//!
//! This module provides a robust HTTP client with:
//! - Automatic retries with exponential backoff
//! - Circuit breaker pattern
//! - Progress tracking for downloads
//! - Connectivity checking
//!
//! # Example
//!
//! ```rust,no_run
//! use storystream_network::{Client, DownloadManager, ProgressTracker};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::new()?;
//! let manager = DownloadManager::new(client);
//!
//! let progress = ProgressTracker::new(None);
//! manager.download_file(
//!     "https://example.com/audiobook.mp3",
//!     "audiobook.mp3",
//!     Some(progress)
//! ).await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod connectivity;
mod download;
mod error;
mod progress;

pub use client::{Client, ClientConfig};
pub use connectivity::ConnectivityChecker;
pub use download::DownloadManager;
pub use error::{NetworkError, NetworkResult};
pub use progress::{DownloadProgress, ProgressTracker};

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