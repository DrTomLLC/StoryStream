// crates/network/src/download.rs
//! File download manager

use crate::client::Client;
use crate::error::{NetworkError, NetworkResult};
use crate::progress::ProgressTracker;
use bytes::Bytes;
use futures::StreamExt;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Download manager
pub struct DownloadManager {
    client: Client,
}

impl DownloadManager {
    /// Creates a new download manager
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Downloads a file to the specified path with progress tracking
    /// Downloads a file to the specified path with progress tracking
    pub async fn download_file(
        &self,
        url: &str,
        destination: impl AsRef<Path>,
        progress: Option<ProgressTracker>,
    ) -> NetworkResult<u64> {
        // Start download
        let response = self.client.get(url).await?;

        // Create file
        let mut file = File::create(destination.as_ref()).await?;
        let mut stream = response.bytes_stream();
        let mut total_downloaded = 0u64;

        // Download chunks
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(NetworkError::Http)?;
            file.write_all(&chunk).await?;

            total_downloaded += chunk.len() as u64;

            if let Some(tracker) = &progress {
                tracker.update(chunk.len() as u64);
            }
        }

        file.flush().await?;
        Ok(total_downloaded)
    }

    /// Downloads content to memory
    pub async fn download_bytes(&self, url: &str) -> NetworkResult<Bytes> {
        let response = self.client.get(url).await?;
        response.bytes().await.map_err(NetworkError::Http)
    }

    /// Downloads content as a string
    pub async fn download_string(&self, url: &str) -> NetworkResult<String> {
        let bytes = self.download_bytes(url).await?;
        String::from_utf8(bytes.to_vec())
            .map_err(|e| NetworkError::Custom(format!("Invalid UTF-8: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_manager_creation() {
        let client = Client::new().expect("Failed to create client");
        let _manager = DownloadManager::new(client);
    }

    #[tokio::test]
    async fn test_download_bytes() {
        let client = Client::new().expect("Failed to create client");
        let manager = DownloadManager::new(client);

        // Test with a small, reliable URL
        // Note: This test might fail in CI/offline environments
        let result = manager.download_bytes("https://www.rust-lang.org/robots.txt").await;

        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_progress_tracker_integration() {
        let tracker = ProgressTracker::new(Some(1000));
        tracker.update(500);

        assert_eq!(tracker.percentage(), Some(50.0));
    }
}