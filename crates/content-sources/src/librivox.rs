// FILE: src/librivox.rs
// ============================================================================

use crate::{ContentSource, SearchQuery, SearchResult, SourceError, SourceMetadata, SourceResult};
use serde::{Deserialize, Serialize};

/// LibriVox content source
pub struct LibriVoxSource {
    base_url: String,
}

impl LibriVoxSource {
    const API_BASE: &'static str = "https://librivox.org/api/feed/audiobooks";

    pub fn new() -> Self {
        Self {
            base_url: Self::API_BASE.to_string(),
        }
    }
}

impl Default for LibriVoxSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentSource for LibriVoxSource {
    fn search(&self, query: &SearchQuery) -> SourceResult<Vec<SearchResult>> {
        // Placeholder - would make HTTP request to LibriVox API
        // Real implementation would use reqwest or similar

        if query.text.is_empty() {
            return Err(SourceError::InvalidQuery("Empty query".to_string()));
        }

        // Mock response for testing
        Ok(Vec::new())
    }

    fn metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: "LibriVox".to_string(),
            description: "Free public domain audiobooks".to_string(),
            base_url: self.base_url.clone(),
            requires_auth: false,
        }
    }

    fn is_available(&self) -> bool {
        // In real implementation, would check network connectivity
        true
    }
}

/// LibriVox book information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibriVoxBook {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub language: String,
    pub url_librivox: String,
}

impl LibriVoxBook {
    pub fn new(id: String, title: String, author: String) -> Self {
        Self {
            id,
            title,
            author,
            description: String::new(),
            language: "en".to_string(),
            url_librivox: String::new(),
        }
    }
}

#[cfg(test)]
mod librivox_tests {
    use super::*;

    #[test]
    fn test_librivox_creation() {
        let source = LibriVoxSource::new();
        assert!(source.is_available());
    }

    #[test]
    fn test_librivox_metadata() {
        let source = LibriVoxSource::new();
        let meta = source.metadata();
        assert_eq!(meta.name, "LibriVox");
        assert!(!meta.requires_auth);
    }

    #[test]
    fn test_empty_query() {
        let source = LibriVoxSource::new();
        let query = SearchQuery::new(String::new());
        let result = source.search(&query);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_query() {
        let source = LibriVoxSource::new();
        let query = SearchQuery::new("Pride and Prejudice".to_string());
        let result = source.search(&query);
        assert!(result.is_ok());
    }

    #[test]
    fn test_librivox_book_creation() {
        let book = LibriVoxBook::new(
            "123".to_string(),
            "Test Book".to_string(),
            "Test Author".to_string(),
        );
        assert_eq!(book.id, "123");
        assert_eq!(book.language, "en");
    }
}
