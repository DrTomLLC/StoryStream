// FILE: src/local.rs
// ============================================================================

use crate::{ContentSource, SearchQuery, SearchResult, SourceError, SourceMetadata, SourceResult};
use std::path::PathBuf;

/// Local file system content source
pub struct LocalSource {
    paths: Vec<PathBuf>,
}

impl LocalSource {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
        }
    }

    pub fn add_path(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

impl Default for LocalSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentSource for LocalSource {
    fn search(&self, _query: &SearchQuery) -> SourceResult<Vec<SearchResult>> {
        // Placeholder - would scan local directories
        Ok(Vec::new())
    }

    fn metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: "Local Files".to_string(),
            description: "Your local audiobook library".to_string(),
            base_url: String::new(),
            requires_auth: false,
        }
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod local_tests {
    use super::*;

    #[test]
    fn test_local_source_creation() {
        let source = LocalSource::new();
        assert!(source.is_available());
        assert_eq!(source.paths().len(), 0);
    }

    #[test]
    fn test_add_path() {
        let mut source = LocalSource::new();
        source.add_path(PathBuf::from("/test"));
        assert_eq!(source.paths().len(), 1);
    }

    #[test]
    fn test_local_metadata() {
        let source = LocalSource::new();
        let meta = source.metadata();
        assert_eq!(meta.name, "Local Files");
        assert!(!meta.requires_auth);
    }
}
