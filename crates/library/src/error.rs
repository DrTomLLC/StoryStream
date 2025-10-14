// FILE: crates/library/src/error.rs

use storystream_core::error::AppError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("Database error: {0}")]
    Database(#[from] AppError),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Book not found: {0}")]
    BookNotFound(String),

    #[error("Invalid file: {0}")]
    InvalidFile(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Metadata extraction failed: {0}")]
    MetadataError(String),

    #[error("Import failed: {0}")]
    ImportFailed(String),

    #[error("Scanner error: {0}")]
    ScannerError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

// Both type aliases for convenience
pub type Result<T> = std::result::Result<T, LibraryError>;
pub type LibraryResult<T> = std::result::Result<T, LibraryError>;