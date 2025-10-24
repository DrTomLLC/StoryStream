// FILE: crates/library/src/metadata.rs

use anyhow::{Context, Result};
use lofty::file::TaggedFile;
use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;
use storystream_core::{Book, Duration};
use storystream_media_formats::{AudioAnalyzer, AudioFormat as MediaFormat, FormatDetector};

/// Audio metadata extractor
pub struct MetadataExtractor {
    analyzer: AudioAnalyzer,
    detector: FormatDetector,
}

/// Extracted metadata from an audio file
#[derive(Debug, Clone)]
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub narrator: Option<String>,
    pub description: Option<String>,
    pub series: Option<String>,
    pub series_position: Option<f32>,
    pub duration: Duration,
    pub file_size: u64,
    pub format: MediaFormat,
    pub bitrate: Option<u32>,
    pub sample_rate: u32,
    pub channels: u8,
    pub cover_art: Option<Vec<u8>>,
}

impl MetadataExtractor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            analyzer: AudioAnalyzer::new()?,
            detector: FormatDetector::new(),
        })
    }

    pub fn extract(&self, path: &Path) -> Result<ExtractedMetadata> {
        if !path.exists() {
            anyhow::bail!("File not found: {}", path.display());
        }

        let metadata = std::fs::metadata(path).context("Failed to read file metadata")?;
        let file_size = metadata.len();

        let format = self
            .detector
            .detect_from_file(path)
            .context("Failed to detect audio format")?;

        let properties = self
            .analyzer
            .analyze(path)
            .context("Failed to analyze audio properties")?;

        let duration = properties
            .duration
            .ok_or_else(|| anyhow::anyhow!("Could not determine audio duration"))?;

        // Convert std::time::Duration to our Duration
        let duration = Duration::from_seconds(duration.as_secs());

        let (title, author, narrator, description, series, series_position, cover_art) =
            self.extract_tags(path)?;

        Ok(ExtractedMetadata {
            title,
            author,
            narrator,
            description,
            series,
            series_position,
            duration,
            file_size,
            format,
            bitrate: properties.bitrate,
            sample_rate: properties.sample_rate,
            channels: properties.channels,
            cover_art,
        })
    }

    fn extract_tags(
        &self,
        path: &Path,
    ) -> Result<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<f32>,
        Option<Vec<u8>>,
    )> {
        let tagged_file = match Probe::open(path)
            .context("Failed to open file for tag reading")?
            .read()
        {
            Ok(file) => file,
            Err(_) => {
                return Ok((None, None, None, None, None, None, None));
            }
        };

        let tag = match tagged_file.primary_tag() {
            Some(t) => t,
            None => {
                return Ok((None, None, None, None, None, None, None));
            }
        };

        let title = tag.title().map(|s| s.to_string());
        let author = tag.artist().map(|s| s.to_string());
        let description = tag.comment().map(|s| s.to_string());

        let narrator = tag
            .get_string(&ItemKey::Composer)
            .or_else(|| tag.get_string(&ItemKey::AlbumArtist))
            .map(|s| s.to_string());

        let series = tag.album().map(|s| s.to_string());

        let series_position = tag.track().or_else(|| tag.disk()).map(|n| n as f32);

        let cover_art = self.extract_cover_art(&tagged_file);

        Ok((
            title,
            author,
            narrator,
            description,
            series,
            series_position,
            cover_art,
        ))
    }

    fn extract_cover_art(&self, tagged_file: &TaggedFile) -> Option<Vec<u8>> {
        let tag = tagged_file.primary_tag()?;
        let picture = tag.pictures().first()?;
        Some(picture.data().to_vec())
    }

    pub fn is_supported(path: &Path) -> bool {
        MediaFormat::from_path(path).is_some()
    }

    pub fn supported_extensions() -> Vec<&'static str> {
        MediaFormat::all()
            .iter()
            .filter_map(|fmt| Some(fmt.extension()))
            .collect()
    }

    pub fn to_book(&self, path: &Path, metadata: ExtractedMetadata) -> Book {
        let title = metadata.title.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });

        let mut book = Book::new(
            title,
            path.to_path_buf(),
            metadata.file_size,
            metadata.duration,
        );

        book.author = metadata.author;
        book.narrator = metadata.narrator;
        book.description = metadata.description;
        book.series = metadata.series;
        book.series_position = metadata.series_position;

        book
    }
}

impl Default for MetadataExtractor {
    fn default() -> Self {
        Self::new().expect("Failed to create MetadataExtractor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extractor_creation() {
        let extractor = MetadataExtractor::new();
        assert!(extractor.is_ok());
    }

    #[test]
    fn test_is_supported() {
        assert!(MetadataExtractor::is_supported(Path::new("test.mp3")));
        assert!(MetadataExtractor::is_supported(Path::new("test.m4b")));
        assert!(MetadataExtractor::is_supported(Path::new("test.flac")));
        assert!(!MetadataExtractor::is_supported(Path::new("test.txt")));
    }

    #[test]
    fn test_supported_extensions() {
        let extensions = MetadataExtractor::supported_extensions();
        assert!(!extensions.is_empty());
        assert!(extensions.contains(&"mp3"));
        assert!(extensions.contains(&"m4b"));
    }

    #[test]
    fn test_extract_nonexistent_file() {
        let extractor = MetadataExtractor::new().expect("Failed to create extractor");
        let result = extractor.extract(Path::new("nonexistent.mp3"));
        assert!(result.is_err());
    }

    #[test]
    fn test_to_book_with_metadata() {
        let extractor = MetadataExtractor::new().expect("Failed to create extractor");
        let path = Path::new("test.mp3");

        let metadata = ExtractedMetadata {
            title: Some("Test Book".to_string()),
            author: Some("Test Author".to_string()),
            narrator: Some("Test Narrator".to_string()),
            description: Some("Test description".to_string()),
            series: Some("Test Series".to_string()),
            series_position: Some(1.0),
            duration: Duration::from_seconds(3600),
            file_size: 1024000,
            format: MediaFormat::Mp3,
            bitrate: Some(128000),
            sample_rate: 44100,
            channels: 2,
            cover_art: None,
        };

        let book = extractor.to_book(path, metadata);
        assert_eq!(book.title, "Test Book");
        assert_eq!(book.author, Some("Test Author".to_string()));
        assert_eq!(book.narrator, Some("Test Narrator".to_string()));
        assert_eq!(book.series, Some("Test Series".to_string()));
        assert_eq!(book.series_position, Some(1.0));
    }

    #[test]
    fn test_to_book_fallback_title() {
        let extractor = MetadataExtractor::new().expect("Failed to create extractor");
        let path = Path::new("my_audiobook.mp3");

        let metadata = ExtractedMetadata {
            title: None,
            author: None,
            narrator: None,
            description: None,
            series: None,
            series_position: None,
            duration: Duration::from_seconds(3600),
            file_size: 1024000,
            format: MediaFormat::Mp3,
            bitrate: Some(128000),
            sample_rate: 44100,
            channels: 2,
            cover_art: None,
        };

        let book = extractor.to_book(path, metadata);
        assert_eq!(book.title, "my_audiobook");
    }

    #[test]
    fn test_extract_tags_no_panic_on_invalid_file() {
        let extractor = MetadataExtractor::new().expect("Failed to create extractor");

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"This is not an audio file")
            .expect("Failed to write");
        temp_file.flush().expect("Failed to flush");

        let result = extractor.extract_tags(temp_file.path());
        assert!(result.is_ok());

        let (title, author, _, _, _, _, _) = result.expect("Should not fail");
        assert!(title.is_none());
        assert!(author.is_none());
    }

    #[test]
    fn test_metadata_extractor_default() {
        let _extractor = MetadataExtractor::default();
    }
}
