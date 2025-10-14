// FILE: crates/content-sources/src/local.rs

use crate::{ContentSource, SearchQuery, SearchResult, SourceMetadata, SourceResult};
use serde::{Deserialize, Serialize};

/// Local filesystem content source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSource {
    /// Root directory for this source
    pub root_path: std::path::PathBuf,
    /// Human-readable name for this source
    pub name: String,
}

impl LocalSource {
    /// Creates a new local source with default values
    pub fn new() -> Self {
        Self {
            name: "Local Files".to_string(),
            root_path: std::path::PathBuf::from("."),
        }
    }

    /// Creates a local source with specific name and path
    pub fn with_path(name: String, root_path: std::path::PathBuf) -> Self {
        Self { name, root_path }
    }
}

impl Default for LocalSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentSource for LocalSource {
    fn metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: self.name.clone(),
            description: format!("Local audiobooks at {}", self.root_path.display()),
            base_url: String::new(),
            requires_auth: false,
        }
    }

    fn search(&self, _query: &SearchQuery) -> SourceResult<Vec<SearchResult>> {
        // TODO: Implement local filesystem search
        Ok(Vec::new())
    }

    fn is_available(&self) -> bool {
        self.root_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_local_source_creation() {
        let source = LocalSource::new();
        assert_eq!(source.name, "Local Files");
    }

    #[test]
    fn test_local_source_with_path() {
        let source = LocalSource::with_path(
            "My Audiobooks".to_string(),
            PathBuf::from("/home/user/audiobooks"),
        );
        assert_eq!(source.name, "My Audiobooks");
        assert_eq!(source.root_path, PathBuf::from("/home/user/audiobooks"));
    }

    #[test]
    fn test_local_source_metadata() {
        let source = LocalSource::new();
        let metadata = source.metadata();
        assert_eq!(metadata.name, "Local Files");
        assert!(!metadata.requires_auth);
    }

    #[test]
    fn test_local_source_is_available() {
        let source = LocalSource::with_path(
            "Test".to_string(),
            PathBuf::from("."), // Current directory exists
        );
        assert!(source.is_available());

        let source = LocalSource::with_path(
            "Test".to_string(),
            PathBuf::from("/nonexistent/path"),
        );
        assert!(!source.is_available());
    }
}