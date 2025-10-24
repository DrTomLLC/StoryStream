// FILE: src/archive.rs
// ============================================================================

use crate::{ContentSource, SearchQuery, SearchResult, SourceError, SourceMetadata, SourceResult};
use serde::{Deserialize, Serialize};

/// Internet Archive content source
pub struct ArchiveSource {
    base_url: String,
}

impl ArchiveSource {
    const API_BASE: &'static str = "https://archive.org/advancedsearch.php";

    pub fn new() -> Self {
        Self {
            base_url: Self::API_BASE.to_string(),
        }
    }
}

impl Default for ArchiveSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentSource for ArchiveSource {
    fn search(&self, query: &SearchQuery) -> SourceResult<Vec<SearchResult>> {
        if query.text.is_empty() {
            return Err(SourceError::InvalidQuery("Empty query".to_string()));
        }

        // Placeholder - would make HTTP request
        Ok(Vec::new())
    }

    fn metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: "Internet Archive".to_string(),
            description: "Large collection of audiobooks and audio content".to_string(),
            base_url: self.base_url.clone(),
            requires_auth: false,
        }
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// Internet Archive item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveItem {
    pub identifier: String,
    pub title: String,
    pub creator: Option<String>,
    pub description: Option<String>,
    pub mediatype: String,
}

impl ArchiveItem {
    pub fn new(identifier: String, title: String) -> Self {
        Self {
            identifier,
            title,
            creator: None,
            description: None,
            mediatype: "audio".to_string(),
        }
    }

    pub fn is_audio(&self) -> bool {
        self.mediatype == "audio" || self.mediatype == "etree"
    }
}

#[cfg(test)]
mod archive_tests {
    use super::*;

    #[test]
    fn test_archive_creation() {
        let source = ArchiveSource::new();
        assert!(source.is_available());
    }

    #[test]
    fn test_archive_metadata() {
        let source = ArchiveSource::new();
        let meta = source.metadata();
        assert_eq!(meta.name, "Internet Archive");
    }

    #[test]
    fn test_empty_query() {
        let source = ArchiveSource::new();
        let query = SearchQuery::new(String::new());
        assert!(source.search(&query).is_err());
    }

    #[test]
    fn test_archive_item_creation() {
        let item = ArchiveItem::new("test-id".to_string(), "Test Item".to_string());
        assert_eq!(item.identifier, "test-id");
        assert!(item.is_audio());
    }

    #[test]
    fn test_archive_item_audio_detection() {
        let mut item = ArchiveItem::new("test".to_string(), "Test".to_string());
        assert!(item.is_audio());

        item.mediatype = "etree".to_string();
        assert!(item.is_audio());

        item.mediatype = "video".to_string();
        assert!(!item.is_audio());
    }
}
