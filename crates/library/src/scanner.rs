// FILE: crates/library/src/scanner.rs

use crate::error::{LibraryError, Result};
use log::{debug, error, info, warn};
use notify::{Error as NotifyError, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::mpsc;
use walkdir::WalkDir;

const DEFAULT_DEBOUNCE_MS: u64 = 500;
const CHANNEL_BUFFER_SIZE: usize = 100;

/// Supported audio file extensions
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "m4a", "m4b", "flac", "ogg", "opus", "aac", "wma", "wav", "aiff", "ape", "wv",
];

/// Configuration for library scanner
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Paths to watch for changes
    pub watch_paths: Vec<String>,
    /// Maximum depth for recursive scanning (0 = unlimited)
    pub max_depth: Option<usize>,
    /// Minimum file size in bytes (files smaller are ignored)
    pub min_file_size: u64,
    /// Follow symbolic links
    pub follow_symlinks: bool,
    /// Supported file extensions (defaults to common audio formats)
    pub supported_extensions: HashSet<String>,
    /// Debounce duration for file system events (milliseconds)
    pub debounce_ms: u64,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            watch_paths: Vec::new(),
            max_depth: Some(10), // Reasonable default to prevent infinite recursion
            min_file_size: 1024, // 1 KB minimum
            follow_symlinks: false,
            supported_extensions: SUPPORTED_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
            debounce_ms: DEFAULT_DEBOUNCE_MS,
        }
    }
}

impl ScannerConfig {
    pub fn new(watch_paths: Vec<String>) -> Self {
        Self {
            watch_paths,
            ..Default::default()
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn with_min_file_size(mut self, size: u64) -> Self {
        self.min_file_size = size;
        self
    }

    pub fn with_follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.supported_extensions = extensions.into_iter().collect();
        self
    }
}

/// Events emitted by the scanner
#[derive(Debug, Clone)]
pub enum ScanEvent {
    /// New file discovered
    FileAdded(PathBuf),
    /// File was modified
    FileModified(PathBuf),
    /// File was removed
    FileRemoved(PathBuf),
    /// Scan completed with file count
    ScanCompleted(usize),
    /// Error occurred during scanning
    ScanError(String),
}

/// Library scanner for monitoring file changes
pub struct LibraryScanner {
    config: ScannerConfig,
    running: Arc<AtomicBool>,
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
    event_tx: Option<mpsc::Sender<ScanEvent>>,
}

impl LibraryScanner {
    /// Create a new scanner with the given watch paths
    pub fn new(watch_paths: Vec<String>) -> Self {
        Self::with_config(ScannerConfig::new(watch_paths))
    }

    /// Create a new scanner with custom configuration
    pub fn with_config(config: ScannerConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            watcher: Arc::new(Mutex::new(None)),
            event_tx: None,
        }
    }

    /// Scan all configured paths and return found audio files
    pub async fn scan(&self) -> Result<Vec<PathBuf>> {
        info!(
            "Starting library scan of {} paths",
            self.config.watch_paths.len()
        );

        let mut found_files = Vec::new();
        let mut scanned_paths = HashSet::new();

        for watch_path in &self.config.watch_paths {
            let path = PathBuf::from(watch_path);

            // Skip if path doesn't exist
            if !path.exists() {
                warn!("Watch path does not exist: {}", watch_path);
                continue;
            }

            // Skip if we've already scanned this path (handles duplicate paths)
            let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
            if scanned_paths.contains(&canonical) {
                debug!("Skipping already scanned path: {}", watch_path);
                continue;
            }
            scanned_paths.insert(canonical);

            // If it's a file, check if it's valid and add it
            if path.is_file() {
                if self.is_valid_audio_file(&path)? {
                    found_files.push(path);
                }
                continue;
            }

            // It's a directory - walk it
            let files = self.scan_directory(&path).await?;
            found_files.extend(files);
        }

        info!("Scan completed: found {} audio files", found_files.len());

        // Send completion event if we have a channel
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(ScanEvent::ScanCompleted(found_files.len())).await;
        }

        Ok(found_files)
    }

    /// Scan a single directory recursively
    async fn scan_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let walker = WalkDir::new(path)
            .follow_links(self.config.follow_symlinks)
            .max_depth(self.config.max_depth.unwrap_or(usize::MAX));

        for entry in walker {
            // Check if we should stop (scanner was stopped)
            if !self.running.load(Ordering::Relaxed) && self.is_running().await {
                // If we're in the middle of a watch operation that got stopped
                break;
            }

            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Error walking directory: {}", e);
                    continue;
                }
            };

            let entry_path = entry.path();

            // Skip if not a file
            if !entry_path.is_file() {
                continue;
            }

            // Check if valid audio file
            match self.is_valid_audio_file(entry_path) {
                Ok(true) => files.push(entry_path.to_path_buf()),
                Ok(false) => {}
                Err(e) => {
                    debug!("Error checking file {}: {}", entry_path.display(), e);
                }
            }

            // Yield to allow other tasks to run periodically
            if files.len() % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }

        Ok(files)
    }

    /// Check if a file is a valid audio file based on extension and size
    fn is_valid_audio_file(&self, path: &Path) -> Result<bool> {
        // Check extension
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let has_valid_extension = extension
            .map(|ext| self.config.supported_extensions.contains(&ext))
            .unwrap_or(false);

        if !has_valid_extension {
            return Ok(false);
        }

        // Check file size
        let metadata = std::fs::metadata(path).map_err(|e| LibraryError::Io(e))?;

        if metadata.len() < self.config.min_file_size {
            return Ok(false);
        }

        Ok(true)
    }

    /// Start watching for file system changes
    pub async fn start(&mut self) -> Result<mpsc::Receiver<ScanEvent>> {
        if self.is_running().await {
            return Err(LibraryError::ScannerError(
                "Scanner is already running".to_string(),
            ));
        }

        info!("Starting file system watcher");

        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        self.event_tx = Some(tx.clone());

        // Create watcher
        let event_tx = tx.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        let mut watcher =
            notify::recommended_watcher(move |res: std::result::Result<Event, NotifyError>| {
                match res {
                    Ok(event) => {
                        if let Err(e) = handle_fs_event(event, &event_tx, &config) {
                            error!("Error handling file system event: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Watch error: {}", e);
                        let _ = event_tx.blocking_send(ScanEvent::ScanError(e.to_string()));
                    }
                }
            })
            .map_err(|e| LibraryError::ScannerError(format!("Failed to create watcher: {}", e)))?;

        // Watch all configured paths
        for watch_path in &self.config.watch_paths {
            let path = PathBuf::from(watch_path);

            if !path.exists() {
                warn!("Skipping non-existent watch path: {}", watch_path);
                continue;
            }

            watcher
                .watch(&path, RecursiveMode::Recursive)
                .map_err(|e| {
                    LibraryError::ScannerError(format!(
                        "Failed to watch path {}: {}",
                        watch_path, e
                    ))
                })?;

            info!("Watching path: {}", watch_path);
        }

        // Store watcher and mark as running
        *self.watcher.lock().unwrap() = Some(watcher);
        running.store(true, Ordering::Relaxed);

        Ok(rx)
    }

    /// Stop watching for file system changes
    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_running().await {
            return Ok(());
        }

        info!("Stopping file system watcher");

        self.running.store(false, Ordering::Relaxed);

        // Drop the watcher to stop receiving events
        *self.watcher.lock().unwrap() = None;

        // Drop the event sender
        self.event_tx = None;

        Ok(())
    }

    /// Check if the scanner is currently running
    pub async fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get the current configuration
    pub fn config(&self) -> &ScannerConfig {
        &self.config
    }
}

/// Handle a file system event and send appropriate scan events
fn handle_fs_event(
    event: Event,
    tx: &mpsc::Sender<ScanEvent>,
    config: &ScannerConfig,
) -> Result<()> {
    match event.kind {
        EventKind::Create(_) => {
            for path in event.paths {
                if is_audio_file(&path, config) {
                    debug!("File added: {}", path.display());
                    let _ = tx.blocking_send(ScanEvent::FileAdded(path));
                }
            }
        }
        EventKind::Modify(_) => {
            for path in event.paths {
                if is_audio_file(&path, config) {
                    debug!("File modified: {}", path.display());
                    let _ = tx.blocking_send(ScanEvent::FileModified(path));
                }
            }
        }
        EventKind::Remove(_) => {
            for path in event.paths {
                // Don't need to check if it's an audio file since it's been removed
                // Just check the extension from the path
                if has_audio_extension(&path, config) {
                    debug!("File removed: {}", path.display());
                    let _ = tx.blocking_send(ScanEvent::FileRemoved(path));
                }
            }
        }
        _ => {
            // Ignore other event types
        }
    }

    Ok(())
}

/// Check if a path is an audio file based on extension
fn is_audio_file(path: &Path, config: &ScannerConfig) -> bool {
    if !path.is_file() {
        return false;
    }

    has_audio_extension(path, config)
}

/// Check if a path has an audio file extension
fn has_audio_extension(path: &Path, config: &ScannerConfig) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| config.supported_extensions.contains(&e.to_lowercase()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_audio_file(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        // Create a file with some content to meet minimum size requirements
        fs::write(&path, vec![0u8; 2048]).unwrap();
        path
    }

    fn create_test_directory_structure(temp_dir: &Path) -> Vec<PathBuf> {
        // Create subdirectories
        fs::create_dir(temp_dir.join("subdir1")).unwrap();
        fs::create_dir(temp_dir.join("subdir1/nested")).unwrap();
        fs::create_dir(temp_dir.join("subdir2")).unwrap();

        // Create audio files
        vec![
            create_test_audio_file(temp_dir, "book1.mp3"),
            create_test_audio_file(temp_dir, "book2.m4b"),
            create_test_audio_file(&temp_dir.join("subdir1"), "book3.flac"),
            create_test_audio_file(&temp_dir.join("subdir1/nested"), "book4.opus"),
            create_test_audio_file(&temp_dir.join("subdir2"), "book5.mp3"),
        ]
    }

    #[test]
    fn test_scanner_creation() {
        let scanner = LibraryScanner::new(vec!["/test".to_string()]);
        assert_eq!(scanner.config.watch_paths.len(), 1);
        assert_eq!(scanner.config.watch_paths[0], "/test");
    }

    #[test]
    fn test_scanner_config_builder() {
        let config = ScannerConfig::new(vec!["/test".to_string()])
            .with_max_depth(5)
            .with_min_file_size(2048)
            .with_follow_symlinks(true);

        assert_eq!(config.max_depth, Some(5));
        assert_eq!(config.min_file_size, 2048);
        assert!(config.follow_symlinks);
    }

    #[test]
    fn test_default_supported_extensions() {
        let config = ScannerConfig::default();
        assert!(config.supported_extensions.contains("mp3"));
        assert!(config.supported_extensions.contains("m4b"));
        assert!(config.supported_extensions.contains("flac"));
        assert!(!config.supported_extensions.contains("txt"));
    }

    #[tokio::test]
    async fn test_scan_empty_directory() -> Result<()> {
        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

        let path = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?
            .to_string();

        let scanner = LibraryScanner::new(vec![path]);
        let files = scanner.scan().await?;
        assert_eq!(files.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_scan_with_audio_files() -> Result<()> {
        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;
        create_test_directory_structure(temp_dir.path());

        let path = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?
            .to_string();

        let scanner = LibraryScanner::new(vec![path]);
        let files = scanner.scan().await?;

        // Should find all 5 audio files
        assert_eq!(files.len(), 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_scan_max_depth() -> Result<()> {
        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;
        create_test_directory_structure(temp_dir.path());

        let path = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?
            .to_string();

        // walkdir max_depth works as follows:
        // depth 0 = root directory itself
        // depth 1 = immediate children of root (both files and subdirs)
        // depth 2 = children of subdirs
        //
        // With max_depth(2): we get root + immediate subdirs + their children
        // This should find files in root and first-level subdirs
        let config = ScannerConfig::new(vec![path]).with_max_depth(2);
        let scanner = LibraryScanner::with_config(config);
        let files = scanner.scan().await?;

        // Should find:
        //   - book1.mp3 (root, depth 1)
        //   - book2.m4b (root, depth 1)
        //   - book3.flac (subdir1, depth 2)
        //   - book5.mp3 (subdir2, depth 2)
        // Should NOT find:
        //   - book4.opus (subdir1/nested, depth 3 - too deep)
        assert_eq!(files.len(), 4);

        Ok(())
    }

    #[tokio::test]
    async fn test_scan_min_file_size() -> Result<()> {
        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

        // Create files of different sizes
        let small_file = temp_dir.path().join("small.mp3");
        fs::write(&small_file, vec![0u8; 512]).unwrap(); // 512 bytes

        let large_file = temp_dir.path().join("large.mp3");
        fs::write(&large_file, vec![0u8; 2048]).unwrap(); // 2048 bytes

        let path = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?
            .to_string();

        // Scan with minimum file size of 1KB
        let scanner = LibraryScanner::new(vec![path]);
        let files = scanner.scan().await?;

        // Should only find the large file
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("large.mp3"));

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
    async fn test_scan_with_non_audio_files() -> Result<()> {
        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

        // Create audio files
        create_test_audio_file(temp_dir.path(), "audio.mp3");

        // Create non-audio files
        let text_file = temp_dir.path().join("readme.txt");
        fs::write(&text_file, "test content").unwrap();

        let image_file = temp_dir.path().join("cover.jpg");
        fs::write(&image_file, vec![0u8; 2048]).unwrap();

        let path = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?
            .to_string();

        let scanner = LibraryScanner::new(vec![path]);
        let files = scanner.scan().await?;

        // Should only find the audio file
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("audio.mp3"));

        Ok(())
    }

    #[tokio::test]
    async fn test_scan_duplicate_paths() -> Result<()> {
        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;
        create_test_audio_file(temp_dir.path(), "audio.mp3");

        let path = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?
            .to_string();

        // Scan with duplicate paths
        let scanner = LibraryScanner::new(vec![path.clone(), path.clone()]);
        let files = scanner.scan().await?;

        // Should only find each file once despite duplicate paths
        assert_eq!(files.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_start_stop_watching() -> Result<()> {
        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

        let path = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?
            .to_string();

        let mut scanner = LibraryScanner::new(vec![path]);

        assert!(!scanner.is_running().await);

        // Start watching
        let _rx = scanner.start().await?;
        assert!(scanner.is_running().await);

        // Stop watching
        scanner.stop().await?;
        assert!(!scanner.is_running().await);

        Ok(())
    }

    #[tokio::test]
    async fn test_start_already_running() -> Result<()> {
        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

        let path = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?
            .to_string();

        let mut scanner = LibraryScanner::new(vec![path]);

        // Start watching
        let _rx = scanner.start().await?;

        // Try to start again - should return error
        let result = scanner.start().await;
        assert!(result.is_err());

        // Cleanup
        scanner.stop().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_watch_file_events() -> Result<()> {
        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

        let path = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?
            .to_string();

        let mut scanner = LibraryScanner::new(vec![path]);
        let mut rx = scanner.start().await?;

        // Give the watcher time to initialize
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create a new audio file
        let test_file = temp_dir.path().join("new_audio.mp3");
        fs::write(&test_file, vec![0u8; 2048]).unwrap();

        // Wait for the event with timeout
        let event = tokio::time::timeout(tokio::time::Duration::from_secs(2), rx.recv()).await;

        scanner.stop().await?;

        // Verify we received an event
        assert!(event.is_ok());
        let event = event.unwrap();
        assert!(event.is_some());

        match event.unwrap() {
            ScanEvent::FileAdded(path) => {
                assert!(path.ends_with("new_audio.mp3"));
            }
            other => panic!("Expected FileAdded event, got {:?}", other),
        }

        Ok(())
    }

    #[test]
    fn test_custom_extensions() {
        let custom_ext = vec!["custom".to_string(), "test".to_string()];
        let config = ScannerConfig::new(vec![]).with_extensions(custom_ext);

        assert!(config.supported_extensions.contains("custom"));
        assert!(config.supported_extensions.contains("test"));
        assert!(!config.supported_extensions.contains("mp3"));
    }
}
