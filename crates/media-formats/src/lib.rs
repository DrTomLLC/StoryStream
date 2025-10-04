extern crate core;

// FILE: lib.rs
mod capabilities;
mod detection;
mod format;
mod mime;

pub use capabilities::{FormatCapabilities, MetadataSupport};
pub use detection::FormatDetector;
pub use format::AudioFormat;
pub use mime::MimeType;

/// Result type for format operations
pub type FormatResult<T> = Result<T, FormatError>;

/// Errors that can occur during format operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// Format could not be determined
    UnknownFormat,
    /// File extension is invalid or empty
    InvalidExtension,
    /// File content does not match expected format
    InvalidMagicBytes,
    /// Format is not supported
    UnsupportedFormat(String),
    /// I/O error occurred
    IoError(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::UnknownFormat => write!(f, "Unknown or unsupported audio format"),
            FormatError::InvalidExtension => write!(f, "Invalid or empty file extension"),
            FormatError::InvalidMagicBytes => {
                write!(f, "File content does not match expected format")
            }
            FormatError::UnsupportedFormat(fmt) => write!(f, "Unsupported format: {}", fmt),
            FormatError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for FormatError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_error_display() {
        let err = FormatError::UnknownFormat;
        assert!(err.to_string().contains("Unknown"));

        let err = FormatError::UnsupportedFormat("xyz".to_string());
        assert!(err.to_string().contains("xyz"));
    }

    #[test]
    fn test_all_modules_compile() {
        // Ensure all modules are accessible
        let _ = AudioFormat::Mp3;
        let _ = FormatCapabilities::default();
        let _ = FormatDetector::new();
        let _ = MimeType::from_format(AudioFormat::Mp3);
    }
}
