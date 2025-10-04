use crate::error::{LibraryError, Result};
use lofty::{
    prelude::{Accessor, AudioFile, TaggedFileExt},
    probe::Probe,
};
use std::path::Path;
use storystream_core::Duration;

/// Extracted audio file metadata
#[derive(Debug, Clone)]
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Duration,
    pub file_size: u64,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub cover_art: Option<Vec<u8>>,
}

/// Extracts metadata from audio files
pub struct AudioMetadataExtractor;

impl AudioMetadataExtractor {
    /// Extract metadata from an audio file
    pub fn extract<P: AsRef<Path>>(path: P) -> Result<ExtractedMetadata> {
        let path = path.as_ref();

        // Verify file exists
        if !path.exists() {
            return Err(LibraryError::FileNotFound(path.display().to_string()));
        }

        // Get file size
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();

        // Probe the file
        let tagged_file = Probe::open(path)
            .map_err(|e| LibraryError::MetadataExtraction(format!("Failed to open file: {}", e)))?
            .read()
            .map_err(|e| {
                LibraryError::MetadataExtraction(format!("Failed to read metadata: {}", e))
            })?;

        // Extract properties
        let properties = tagged_file.properties();
        let duration_ms = properties.duration().as_millis() as u64;
        let duration = Duration::from_millis(duration_ms);

        let bitrate = properties.audio_bitrate();
        let sample_rate = properties.sample_rate();
        let channels = properties.channels().map(|c| c as u8);

        // Extract tags
        let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

        let title = tag.and_then(|t| t.title().map(|s| s.to_string()));
        let artist = tag.and_then(|t| t.artist().map(|s| s.to_string()));
        let album = tag.and_then(|t| t.album().map(|s| s.to_string()));

        // Extract cover art
        let cover_art = tag
            .and_then(|t| t.pictures().first())
            .map(|pic| pic.data().to_vec());

        Ok(ExtractedMetadata {
            title,
            artist,
            album,
            duration,
            file_size,
            bitrate,
            sample_rate,
            channels,
            cover_art,
        })
    }

    /// Check if a file is a supported audio format
    pub fn is_supported<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                matches!(
                    ext.to_lowercase().as_str(),
                    "mp3" | "m4a" | "m4b" | "ogg" | "opus" | "flac" | "wav"
                )
            })
            .unwrap_or(false)
    }

    /// Get supported file extensions
    pub fn supported_extensions() -> &'static [&'static str] {
        &["mp3", "m4a", "m4b", "ogg", "opus", "flac", "wav"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported() {
        assert!(AudioMetadataExtractor::is_supported("book.mp3"));
        assert!(AudioMetadataExtractor::is_supported("book.m4b"));
        assert!(AudioMetadataExtractor::is_supported("book.M4B"));
        assert!(!AudioMetadataExtractor::is_supported("book.txt"));
        assert!(!AudioMetadataExtractor::is_supported("book"));
    }

    #[test]
    fn test_supported_extensions() {
        let exts = AudioMetadataExtractor::supported_extensions();
        assert!(exts.contains(&"mp3"));
        assert!(exts.contains(&"m4b"));
        assert!(exts.len() > 0);
    }

    #[test]
    fn test_extract_nonexistent_file() {
        let result = AudioMetadataExtractor::extract("/nonexistent/file.mp3");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LibraryError::FileNotFound(_)));
    }
}