use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("Database error: {0}")]
    Database(#[from] storystream_core::AppError),

    #[error("Metadata extraction failed: {0}")]
    MetadataExtraction(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid file: {0}")]
    InvalidFile(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Media engine error: {0}")]
    MediaEngine(#[from] media_engine::EngineError),

    #[error("Book not found: {0}")]
    BookNotFound(String),

    #[error("Import failed: {0}")]
    ImportFailed(String),

    #[error("Scanner error: {0}")]
    ScannerError(String),
}

pub type Result<T> = std::result::Result<T, LibraryError>;