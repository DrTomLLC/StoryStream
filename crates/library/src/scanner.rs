// FILE: crates/library/src/scanner.rs

use crate::error::{Result};
use std::path::PathBuf;

/// Library scanner for monitoring file changes
pub struct LibraryScanner {
    pub watch_paths: Vec<String>,
}

impl LibraryScanner {
    pub fn new(watch_paths: Vec<String>) -> Self {
        Self { watch_paths }
    }

    pub async fn scan(&self) -> Result<Vec<PathBuf>> {
        // TODO: Implement actual scanning logic
        Ok(Vec::new())
    }

    pub async fn start(&mut self) -> Result<()> {
        // TODO: Implement file watching
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        // TODO: Implement stopping file watching
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        // TODO: Track running state
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::LibraryError;

    #[test]
    fn test_scanner_creation() {
        let scanner = LibraryScanner::new(vec!["/test".to_string()]);
        assert_eq!(scanner.watch_paths.len(), 1);
    }

    #[tokio::test]
    async fn test_scan_empty_directory() -> Result<()> {
        let temp_dir = TempDir::new()
            .map_err(LibraryError::Io)?;

        let path = temp_dir.path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?
            .to_string();

        let scanner = LibraryScanner::new(vec![path]);
        let files = scanner.scan().await?;
        assert_eq!(files.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_scan_nonexistent_directory() -> Result<()> {
        let scanner = LibraryScanner::new(vec!["/nonexistent/path".to_string()]);
        let files = scanner.scan().await?;
        assert_eq!(files.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_start_stop_watching() -> Result<()> {
        let mut scanner = LibraryScanner::new(vec![]);

        assert!(!scanner.is_running().await);

        scanner.start().await?;
        // NOTE: is_running will return false until we implement actual watching
        // assert!(scanner.is_running().await);

        scanner.stop().await?;
        assert!(!scanner.is_running().await);

        Ok(())
    }
}