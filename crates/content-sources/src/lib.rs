// FILE: crates/content-sources/src/lib.rs

mod archive;
mod librivox;
mod local;
mod traits;

pub use archive::{ArchiveItem, ArchiveSource};
pub use librivox::{LibriVoxBook, LibriVoxSource};
pub use local::LocalSource;
use std::fmt;
pub use traits::{ContentSource, SearchQuery, SearchResult, SourceMetadata};

/// Result type for content source operations
pub type SourceResult<T> = Result<T, SourceError>;

/// Errors from content sources
#[derive(Debug, Clone, PartialEq)]
pub enum SourceError {
    /// Network error
    NetworkError(String),
    /// Parse error
    ParseError(String),
    /// Not found
    NotFound,
    /// Invalid query
    InvalidQuery(String),
    /// Rate limited
    RateLimited,
    /// Source unavailable
    Unavailable(String),
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceError::NetworkError(e) => write!(f, "Network error: {}", e),
            SourceError::ParseError(e) => write!(f, "Parse error: {}", e),
            SourceError::NotFound => write!(f, "Not found"),
            SourceError::InvalidQuery(e) => write!(f, "Invalid query: {}", e),
            SourceError::RateLimited => write!(f, "Rate limited"),
            SourceError::Unavailable(e) => write!(f, "Source unavailable: {}", e),
        }
    }
}

impl std::error::Error for SourceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SourceError::NotFound;
        assert!(err.to_string().contains("Not found"));
    }

    #[test]
    fn test_all_sources_exported() {
        let _ = LocalSource::new(); // Fixed: new() takes no arguments
        let _ = LibriVoxSource::new();
        let _ = ArchiveSource::new();
    }
}
