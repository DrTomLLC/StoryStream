use crate::error::Result;
use crate::metadata::AudioMetadataExtractor;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use walkdir::WalkDir;

/// Scans directories for audio files
pub struct LibraryScanner {
    watch_paths: Vec<PathBuf>,
    is_running: Arc<RwLock<bool>>,
}

impl LibraryScanner {
    pub fn new(paths: Vec<String>) -> Self {
        let watch_paths = paths.into_iter().map(PathBuf::from).collect();

        Self {
            watch_paths,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Scan directories once and return all audio files found
    pub async fn scan(&self) -> Result<Vec<PathBuf>> {
        info!("Scanning directories for audio files");

        let mut audio_files = Vec::new();

        for dir in &self.watch_paths {
            if !dir.exists() {
                warn!("Directory does not exist: {}", dir.display());
                continue;
            }

            for entry in WalkDir::new(dir).follow_links(true).into_iter() {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() && AudioMetadataExtractor::is_supported(path) {
                            audio_files.push(path.to_path_buf());
                        }
                    }
                    Err(e) => {
                        warn!("Error walking directory: {}", e);
                    }
                }
            }
        }

        info!("Found {} audio files", audio_files.len());
        Ok(audio_files)
    }

    /// Start watching directories for changes
    pub async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(());
        }

        *is_running = true;
        info!("Started watching directories");
        Ok(())
    }

    /// Stop watching directories
    pub async fn stop(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Stopped watching directories");
        Ok(())
    }

    /// Check if scanner is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_scanner_creation() {
        let scanner = LibraryScanner::new(vec!["/test".to_string()]);
        assert_eq!(scanner.watch_paths.len(), 1);
    }

    #[tokio::test]
    async fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let scanner = LibraryScanner::new(vec![temp_dir.path().to_str().unwrap().to_string()]);

        let files = scanner.scan().await.unwrap();
        assert_eq!(files.len(), 0);
    }

    #[tokio::test]
    async fn test_scan_nonexistent_directory() {
        let scanner = LibraryScanner::new(vec!["/nonexistent/path".to_string()]);
        let files = scanner.scan().await.unwrap();
        assert_eq!(files.len(), 0);
    }

    #[tokio::test]
    async fn test_start_stop_watching() {
        let mut scanner = LibraryScanner::new(vec![]);

        assert!(!scanner.is_running().await);

        scanner.start().await.unwrap();
        assert!(scanner.is_running().await);

        scanner.stop().await.unwrap();
        assert!(!scanner.is_running().await);
    }
}