// crates/feed-parser/src/error.rs
//! Error types for feed parsing

use thiserror::Error;

/// Result type for feed parser operations
pub type FeedResult<T> = Result<T, FeedError>;

/// Errors that can occur during feed parsing
#[derive(Debug, Error)]
pub enum FeedError {
    /// Invalid XML structure
    #[error("Invalid XML: {0}")]
    InvalidXml(String),

    /// Unsupported feed format
    #[error("Unsupported feed format: {0}")]
    UnsupportedFormat(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid date format
    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// XML parsing error
    #[error("XML parsing error: {0}")]
    XmlParse(String),
}

impl From<quick_xml::Error> for FeedError {
    fn from(err: quick_xml::Error) -> Self {
        FeedError::XmlParse(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = FeedError::InvalidXml("test".to_string());
        assert!(format!("{}", err).contains("Invalid XML"));
    }

    #[test]
    fn test_missing_field_error() {
        let err = FeedError::MissingField("title".to_string());
        assert!(format!("{}", err).contains("title"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let feed_err = FeedError::from(io_err);
        assert!(matches!(feed_err, FeedError::Io(_)));
    }
}