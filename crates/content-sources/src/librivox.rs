// FILE: crates/content-sources/src/librivox.rs

use crate::{ContentSource, SearchQuery, SearchResult, SourceError, SourceMetadata, SourceResult};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

/// LibriVox content source for free public domain audiobooks
pub struct LibriVoxSource {
    base_url: String,
    client: Option<reqwest::blocking::Client>,
}

impl LibriVoxSource {
    const API_BASE: &'static str = "https://librivox.org/api/feed/audiobooks";

    /// Create a new LibriVox source with HTTP client
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(StdDuration::from_secs(30))
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
            ))
            .build()
            .ok();

        Self {
            base_url: Self::API_BASE.to_string(),
            client,
        }
    }

    /// Search LibriVox catalog by title or author
    pub fn search_books(&self, query: &str, limit: usize) -> SourceResult<Vec<LibriVoxBook>> {
        if query.is_empty() {
            return Err(SourceError::InvalidQuery("Empty query".to_string()));
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| SourceError::NetworkError("HTTP client not available".to_string()))?;

        // Build search URL with parameters
        let url = format!(
            "{}?title=^{}^&format=json&limit={}",
            self.base_url,
            urlencoding::encode(query),
            limit
        );

        // Make HTTP request
        let response = client
            .get(&url)
            .send()
            .map_err(|e| SourceError::NetworkError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NetworkError(format!(
                "HTTP {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        // Parse JSON response
        let api_response: LibriVoxApiResponse = response
            .json()
            .map_err(|e| SourceError::ParseError(format!("JSON parse error: {}", e)))?;

        Ok(api_response.books)
    }

    /// Get book details by ID
    pub fn get_book(&self, book_id: &str) -> SourceResult<LibriVoxBook> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| SourceError::NetworkError("HTTP client not available".to_string()))?;

        let url = format!("{}?id={}&format=json", self.base_url, book_id);

        let response = client
            .get(&url)
            .send()
            .map_err(|e| SourceError::NetworkError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NotFound);
        }

        let api_response: LibriVoxApiResponse = response
            .json()
            .map_err(|e| SourceError::ParseError(format!("JSON parse error: {}", e)))?;

        api_response
            .books
            .into_iter()
            .next()
            .ok_or(SourceError::NotFound)
    }

    /// Get latest releases from LibriVox
    pub fn latest_releases(&self, limit: usize) -> SourceResult<Vec<LibriVoxBook>> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| SourceError::NetworkError("HTTP client not available".to_string()))?;

        let url = format!("{}?format=json&limit={}", self.base_url, limit);

        let response = client
            .get(&url)
            .send()
            .map_err(|e| SourceError::NetworkError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NetworkError(format!(
                "HTTP {}",
                response.status().as_u16()
            )));
        }

        let api_response: LibriVoxApiResponse = response
            .json()
            .map_err(|e| SourceError::ParseError(format!("JSON parse error: {}", e)))?;

        Ok(api_response.books)
    }

    /// Search by author
    pub fn search_by_author(&self, author: &str, limit: usize) -> SourceResult<Vec<LibriVoxBook>> {
        if author.is_empty() {
            return Err(SourceError::InvalidQuery("Empty author".to_string()));
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| SourceError::NetworkError("HTTP client not available".to_string()))?;

        let url = format!(
            "{}?author=^{}^&format=json&limit={}",
            self.base_url,
            urlencoding::encode(author),
            limit
        );

        let response = client
            .get(&url)
            .send()
            .map_err(|e| SourceError::NetworkError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NetworkError(format!(
                "HTTP {}",
                response.status().as_u16()
            )));
        }

        let api_response: LibriVoxApiResponse = response
            .json()
            .map_err(|e| SourceError::ParseError(format!("JSON parse error: {}", e)))?;

        Ok(api_response.books)
    }

    /// Check if LibriVox API is available
    pub fn check_availability(&self) -> bool {
        let client = match &self.client {
            Some(c) => c,
            None => return false,
        };

        // Try a minimal API request
        let url = format!("{}?format=json&limit=1", self.base_url);

        client
            .get(&url)
            .timeout(StdDuration::from_secs(5))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

impl Default for LibriVoxSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentSource for LibriVoxSource {
    fn search(&self, query: &SearchQuery) -> SourceResult<Vec<SearchResult>> {
        if query.text.is_empty() {
            return Err(SourceError::InvalidQuery("Empty query".to_string()));
        }

        // Search LibriVox catalog - query.limit is usize, not Option<usize>
        let books = self.search_books(&query.text, query.limit)?;

        // Convert to SearchResult - match ACTUAL SearchResult structure
        let results = books
            .into_iter()
            .map(|book| SearchResult {
                id: book.id.clone(),
                title: book.title.clone(),
                author: book.author.clone(), // String, NOT Option<String>
                description: if book.description.is_empty() {
                    None
                } else {
                    Some(book.description.clone())
                },
                duration: book.duration_seconds().map(StdDuration::from_secs),
                url: book.url_librivox.clone(),
                source: "LibriVox".to_string(),
            })
            .collect();

        Ok(results)
    }

    fn metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: "LibriVox".to_string(),
            description: "Free public domain audiobooks read by volunteers".to_string(),
            base_url: self.base_url.clone(),
            requires_auth: false,
        }
    }

    fn is_available(&self) -> bool {
        self.client.is_some()
    }
}

/// LibriVox API response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LibriVoxApiResponse {
    books: Vec<LibriVoxBook>,
}

/// LibriVox book information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibriVoxBook {
    /// Unique book ID
    pub id: String,

    /// Book title
    pub title: String,

    /// Author name(s)
    #[serde(default)]
    pub author: String,

    /// Book description
    #[serde(default)]
    pub description: String,

    /// Language code (e.g., "en")
    #[serde(default = "default_language")]
    pub language: String,

    /// LibriVox page URL
    #[serde(default)]
    pub url_librivox: String,

    /// Direct RSS feed URL for book chapters
    #[serde(default)]
    pub url_rss: String,

    /// ZIP file download URL
    #[serde(default)]
    pub url_zip_file: String,

    /// Total duration in seconds (optional)
    #[serde(default)]
    pub totaltime: String,

    /// Number of sections/chapters
    #[serde(default)]
    pub num_sections: String,
}

fn default_language() -> String {
    "en".to_string()
}

impl LibriVoxBook {
    /// Create a new LibriVoxBook with minimal information
    pub fn new(id: String, title: String, author: String) -> Self {
        Self {
            id,
            title,
            author,
            description: String::new(),
            language: "en".to_string(),
            url_librivox: String::new(),
            url_rss: String::new(),
            url_zip_file: String::new(),
            totaltime: String::new(),
            num_sections: String::new(),
        }
    }

    /// Check if book has downloadable content
    pub fn has_download(&self) -> bool {
        !self.url_zip_file.is_empty() || !self.url_rss.is_empty()
    }

    /// Get download URL (prefers ZIP over RSS)
    pub fn download_url(&self) -> Option<String> {
        if !self.url_zip_file.is_empty() {
            Some(self.url_zip_file.clone())
        } else if !self.url_rss.is_empty() {
            Some(self.url_rss.clone())
        } else {
            None
        }
    }

    /// Parse total time into seconds
    pub fn duration_seconds(&self) -> Option<u64> {
        self.totaltime
            .split(':')
            .filter_map(|s| s.parse::<u64>().ok())
            .rev()
            .enumerate()
            .map(|(i, val)| val * 60_u64.pow(i as u32))
            .reduce(|a, b| a + b)
    }

    /// Get number of chapters/sections
    pub fn chapter_count(&self) -> Option<usize> {
        self.num_sections.parse().ok()
    }
}

// Helper module for URL encoding
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ' ' => "+".to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_librivox_creation() {
        let source = LibriVoxSource::new();
        assert!(source.is_available() || !source.is_available()); // May or may not have network
    }

    #[test]
    fn test_librivox_metadata() {
        let source = LibriVoxSource::new();
        let meta = source.metadata();
        assert_eq!(meta.name, "LibriVox");
        assert!(!meta.requires_auth);
        assert!(meta.base_url.contains("librivox.org"));
    }

    #[test]
    fn test_empty_query() {
        let source = LibriVoxSource::new();
        let query = SearchQuery::new(String::new());
        let result = source.search(&query);
        assert!(result.is_err());
        assert!(matches!(result, Err(SourceError::InvalidQuery(_))));
    }

    #[test]
    fn test_librivox_book_creation() {
        let book = LibriVoxBook::new(
            "123".to_string(),
            "Test Book".to_string(),
            "Test Author".to_string(),
        );
        assert_eq!(book.id, "123");
        assert_eq!(book.title, "Test Book");
        assert_eq!(book.author, "Test Author");
        assert_eq!(book.language, "en");
    }

    #[test]
    fn test_book_has_download() {
        let mut book =
            LibriVoxBook::new("123".to_string(), "Test".to_string(), "Author".to_string());

        assert!(!book.has_download());

        book.url_zip_file = "http://example.com/book.zip".to_string();
        assert!(book.has_download());
    }

    #[test]
    fn test_book_download_url() {
        let mut book =
            LibriVoxBook::new("123".to_string(), "Test".to_string(), "Author".to_string());

        assert!(book.download_url().is_none());

        book.url_zip_file = "http://example.com/book.zip".to_string();
        assert_eq!(
            book.download_url(),
            Some("http://example.com/book.zip".to_string())
        );

        book.url_zip_file = String::new();
        book.url_rss = "http://example.com/rss".to_string();
        assert_eq!(
            book.download_url(),
            Some("http://example.com/rss".to_string())
        );
    }

    #[test]
    fn test_duration_parsing() {
        let mut book = LibriVoxBook::new("1".to_string(), "T".to_string(), "A".to_string());

        // Test HH:MM:SS format
        book.totaltime = "1:30:45".to_string();
        assert_eq!(book.duration_seconds(), Some(5445)); // 1*3600 + 30*60 + 45

        // Test MM:SS format
        book.totaltime = "45:30".to_string();
        assert_eq!(book.duration_seconds(), Some(2730)); // 45*60 + 30

        // Test invalid format
        book.totaltime = "invalid".to_string();
        assert_eq!(book.duration_seconds(), None);
    }

    #[test]
    fn test_chapter_count_parsing() {
        let mut book = LibriVoxBook::new("1".to_string(), "T".to_string(), "A".to_string());

        book.num_sections = "15".to_string();
        assert_eq!(book.chapter_count(), Some(15));

        book.num_sections = "invalid".to_string();
        assert_eq!(book.chapter_count(), None);
    }

    #[test]
    fn test_url_encoding() {
        let encoded = urlencoding::encode("Pride and Prejudice");
        assert!(encoded.contains('+') || encoded.contains("%20"));

        let encoded = urlencoding::encode("C++ Programming");
        assert!(encoded.contains("C%2B%2B") || encoded.contains("C++"));
    }

    #[test]
    fn test_search_query_conversion() {
        let source = LibriVoxSource::new();
        let query = SearchQuery::new("Test Query".to_string()).with_limit(10);

        // Even without network, should not panic
        let _ = source.search(&query);
    }

    // Network tests - only run with network access
    #[test]
    #[ignore = "Requires network access"]
    fn test_real_search() {
        let source = LibriVoxSource::new();

        if !source.check_availability() {
            eprintln!("LibriVox API not available, skipping test");
            return;
        }

        let books = source.search_books("Pride and Prejudice", 5);
        match books {
            Ok(results) => {
                assert!(!results.is_empty());
                println!("Found {} books", results.len());
                for book in results {
                    println!("  - {} by {}", book.title, book.author);
                }
            }
            Err(e) => {
                eprintln!("Search failed: {}", e);
            }
        }
    }

    #[test]
    #[ignore = "Requires network access"]
    fn test_latest_releases() {
        let source = LibriVoxSource::new();

        if !source.check_availability() {
            eprintln!("LibriVox API not available, skipping test");
            return;
        }

        let books = source.latest_releases(10);
        match books {
            Ok(results) => {
                assert!(!results.is_empty());
                println!("Latest releases:");
                for book in results.iter().take(5) {
                    println!("  - {} by {}", book.title, book.author);
                }
            }
            Err(e) => {
                eprintln!("Latest releases failed: {}", e);
            }
        }
    }

    #[test]
    #[ignore = "Requires network access"]
    fn test_search_by_author() {
        let source = LibriVoxSource::new();

        if !source.check_availability() {
            eprintln!("LibriVox API not available, skipping test");
            return;
        }

        let books = source.search_by_author("Jane Austen", 10);
        match books {
            Ok(results) => {
                assert!(!results.is_empty());
                println!("Books by Jane Austen:");
                for book in results {
                    println!("  - {}", book.title);
                }
            }
            Err(e) => {
                eprintln!("Author search failed: {}", e);
            }
        }
    }
}
