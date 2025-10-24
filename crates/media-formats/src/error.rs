//! Error types for media format operations

use std::path::PathBuf;

/// Result type for format operations
pub type FormatResult<T> = Result<T, FormatError>;

/// Errors that can occur during format detection and analysis
#[derive(Debug, Clone, PartialEq)] // Added PartialEq
pub enum FormatError {
    // Old variants that existing code expects
    /// Format could not be determined
    UnknownFormat,
    /// Invalid file extension
    InvalidExtension,
    /// Invalid magic bytes
    InvalidMagicBytes,
    /// Unsupported format
    UnsupportedFormat(String),
    /// I/O error
    IoError(String),

    // New detailed variants
    /// File not found or inaccessible
    FileNotFound { path: PathBuf },
    /// Failed to read file
    ReadError { path: PathBuf, reason: String },
    /// Unsupported or unrecognized format with path
    UnsupportedFormatWithPath { format: String, path: PathBuf },
    /// No decoder available for format
    NoDecoderAvailable { format: String },
    /// File is corrupted or invalid
    CorruptedFile { path: PathBuf, reason: String },
    /// Failed to probe file format
    ProbeError { path: PathBuf, reason: String },
    /// Failed to parse codec parameters
    CodecError { reason: String },
    /// Invalid audio properties
    InvalidProperties { field: String, value: String },
    /// Symphonia decode error
    DecodeError(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFormat => write!(f, "Unknown or unsupported audio format"),
            Self::InvalidExtension => write!(f, "Invalid or empty file extension"),
            Self::InvalidMagicBytes => write!(f, "File content does not match expected format"),
            Self::UnsupportedFormat(fmt) => write!(f, "Unsupported format: {}", fmt),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::FileNotFound { path } => write!(f, "File not found: {}", path.display()),
            Self::ReadError { path, reason } => {
                write!(f, "Failed to read file {}: {}", path.display(), reason)
            }
            Self::UnsupportedFormatWithPath { format, path } => {
                write!(
                    f,
                    "Unsupported audio format: {} in file {}",
                    format,
                    path.display()
                )
            }
            Self::NoDecoderAvailable { format } => {
                write!(f, "No decoder available for format {}", format)
            }
            Self::CorruptedFile { path, reason } => {
                write!(
                    f,
                    "Corrupted or invalid audio file: {} - {}",
                    path.display(),
                    reason
                )
            }
            Self::ProbeError { path, reason } => {
                write!(
                    f,
                    "Failed to probe file format for {}: {}",
                    path.display(),
                    reason
                )
            }
            Self::CodecError { reason } => {
                write!(f, "Failed to parse codec parameters: {}", reason)
            }
            Self::InvalidProperties { field, value } => {
                write!(f, "Invalid audio properties: {} = {}", field, value)
            }
            Self::DecodeError(msg) => write!(f, "Decode error: {}", msg),
        }
    }
}

impl std::error::Error for FormatError {}

impl FormatError {
    pub fn file_not_found(path: PathBuf) -> Self {
        Self::FileNotFound { path }
    }

    pub fn read_error(path: PathBuf, reason: impl Into<String>) -> Self {
        Self::ReadError {
            path,
            reason: reason.into(),
        }
    }

    pub fn unsupported(format: impl Into<String>, path: PathBuf) -> Self {
        Self::UnsupportedFormatWithPath {
            format: format.into(),
            path,
        }
    }

    pub fn corrupted(path: PathBuf, reason: impl Into<String>) -> Self {
        Self::CorruptedFile {
            path,
            reason: reason.into(),
        }
    }

    pub fn probe_error(path: PathBuf, reason: impl Into<String>) -> Self {
        Self::ProbeError {
            path,
            reason: reason.into(),
        }
    }

    pub fn codec_error(reason: impl Into<String>) -> Self {
        Self::CodecError {
            reason: reason.into(),
        }
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::FileNotFound { .. }
                | Self::UnknownFormat
                | Self::UnsupportedFormat(_)
                | Self::UnsupportedFormatWithPath { .. }
        )
    }

    pub fn is_corruption(&self) -> bool {
        matches!(self, Self::CorruptedFile { .. })
    }
}
