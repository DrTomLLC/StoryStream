//! StoryStream Library Management
//!
//! High-level orchestration layer that coordinates core, database, and media-engine.
//! Provides business logic for book management, import, and playback.

pub mod error;
pub mod import;
pub mod manager;
pub mod metadata;
pub mod scanner;

pub use error::{LibraryError, LibraryResult};
pub use import::{BookImporter, ImportOptions};
pub use manager::{LibraryConfig as OtherLibraryConfig, LibraryManager};
pub use metadata::MetadataExtractor;
pub use scanner::LibraryScanner;

/// Library configuration
#[derive(Debug, Clone)]
pub struct LibraryConfig {
    /// Database file path
    pub database_path: String,
    /// Watch directories for new books
    pub watch_directories: Vec<String>,
    /// Automatically import new files
    pub auto_import: bool,
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self {
            database_path: "storystream.db".to_string(),
            watch_directories: Vec::new(),
            auto_import: false,
        }
    }
}

impl LibraryConfig {
    pub fn new(database_path: impl Into<String>) -> Self {
        Self {
            database_path: database_path.into(),
            ..Default::default()
        }
    }

    pub fn with_watch_directory(mut self, path: impl Into<String>) -> Self {
        self.watch_directories.push(path.into());
        self
    }

    pub fn with_auto_import(mut self, enabled: bool) -> Self {
        self.auto_import = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = LibraryConfig::default();
        assert_eq!(config.database_path, "storystream.db");
        assert!(config.watch_directories.is_empty());
        assert!(!config.auto_import);
    }

    #[test]
    fn test_config_builder() {
        let config = LibraryConfig::new("custom.db")
            .with_watch_directory("/audiobooks")
            .with_watch_directory("/podcasts")
            .with_auto_import(true);

        assert_eq!(config.database_path, "custom.db");
        assert_eq!(config.watch_directories.len(), 2);
        assert!(config.auto_import);
    }
}