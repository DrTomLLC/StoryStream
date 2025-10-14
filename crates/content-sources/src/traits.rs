// FILE: src/traits.rs
// ============================================================================

use crate::SourceResult;

/// Content source trait
pub trait ContentSource: Send + Sync {
    /// Search for content
    fn search(&self, query: &SearchQuery) -> SourceResult<Vec<SearchResult>>;

    /// Get metadata about the source
    fn metadata(&self) -> SourceMetadata;

    /// Check if source is available
    fn is_available(&self) -> bool;
}

/// Search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub author: Option<String>,
    pub title: Option<String>,
    pub limit: usize,
}

impl SearchQuery {
    pub fn new(text: String) -> Self {
        Self {
            text,
            author: None,
            title: None,
            limit: 20,
        }
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub duration: Option<std::time::Duration>,
    pub url: String,
    pub source: String,
}

/// Source metadata
#[derive(Debug, Clone)]
pub struct SourceMetadata {
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub requires_auth: bool,
}

#[cfg(test)]
mod trait_tests {
    use super::*;

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("test".to_string())
            .with_author("Author".to_string())
            .with_limit(10);

        assert_eq!(query.text, "test");
        assert_eq!(query.author, Some("Author".to_string()));
        assert_eq!(query.limit, 10);
    }

    #[test]
    fn test_search_query_default() {
        let query = SearchQuery::new("test".to_string());
        assert_eq!(query.limit, 20);
        assert_eq!(query.author, None);
    }
}
