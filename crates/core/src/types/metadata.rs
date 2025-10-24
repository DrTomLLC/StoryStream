//! Audio format and metadata domain models

use crate::types::Duration;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported audio formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioFormat {
    Mp3,
    M4b,
    M4a,
    Flac,
    Ogg,
    Opus,
    Aac,
    Wav,
    Aiff,
}

impl AudioFormat {
    /// Detects format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(Self::Mp3),
            "m4b" => Some(Self::M4b),
            "m4a" => Some(Self::M4a),
            "flac" => Some(Self::Flac),
            "ogg" => Some(Self::Ogg),
            "opus" => Some(Self::Opus),
            "aac" => Some(Self::Aac),
            "wav" => Some(Self::Wav),
            "aiff" | "aif" => Some(Self::Aiff),
            _ => None,
        }
    }

    /// Detects format from file path
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Returns the canonical file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::M4b => "m4b",
            Self::M4a => "m4a",
            Self::Flac => "flac",
            Self::Ogg => "ogg",
            Self::Opus => "opus",
            Self::Aac => "aac",
            Self::Wav => "wav",
            Self::Aiff => "aiff",
        }
    }

    /// Returns the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Mp3 => "audio/mpeg",
            Self::M4b => "audio/mp4",
            Self::M4a => "audio/mp4",
            Self::Flac => "audio/flac",
            Self::Ogg => "audio/ogg",
            Self::Opus => "audio/opus",
            Self::Aac => "audio/aac",
            Self::Wav => "audio/wav",
            Self::Aiff => "audio/aiff",
        }
    }

    /// Returns true if this format supports chapters
    pub fn supports_chapters(&self) -> bool {
        matches!(self, Self::Mp3 | Self::M4b | Self::M4a | Self::Flac)
    }

    /// Returns true if this format supports embedded cover art
    pub fn supports_cover_art(&self) -> bool {
        matches!(
            self,
            Self::Mp3 | Self::M4b | Self::M4a | Self::Flac | Self::Ogg
        )
    }
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension().to_uppercase())
    }
}

/// Audio metadata extracted from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub format: AudioFormat,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub duration: Duration,
    pub bitrate: Option<u32>,     // bits per second
    pub sample_rate: Option<u32>, // Hz
    pub channels: Option<u8>,     // 1=mono, 2=stereo
    pub comment: Option<String>,
    pub has_cover_art: bool,
}

impl AudioMetadata {
    /// Creates new metadata with required fields
    pub fn new(format: AudioFormat, duration: Duration) -> Self {
        Self {
            format,
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            composer: None,
            genre: None,
            year: None,
            track_number: None,
            disc_number: None,
            duration,
            bitrate: None,
            sample_rate: None,
            channels: None,
            comment: None,
            has_cover_art: false,
        }
    }

    /// Returns true if this metadata has basic audio info
    pub fn has_technical_info(&self) -> bool {
        self.bitrate.is_some() && self.sample_rate.is_some() && self.channels.is_some()
    }

    /// Returns a human-readable technical summary
    pub fn technical_summary(&self) -> Option<String> {
        if !self.has_technical_info() {
            return None;
        }

        let bitrate = self.bitrate.unwrap() / 1000; // Convert to kbps
        let sample_rate = self.sample_rate.unwrap() / 1000; // Convert to kHz
        let channels = match self.channels.unwrap() {
            1 => "Mono",
            2 => "Stereo",
            _ => "Multi-channel",
        };

        Some(format!(
            "{} kbps, {} kHz, {}",
            bitrate, sample_rate, channels
        ))
    }
}

/// Cover art data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverArt {
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl CoverArt {
    /// Creates new cover art
    pub fn new(data: Vec<u8>, mime_type: String) -> Self {
        Self { data, mime_type }
    }

    /// Returns the image format from MIME type
    pub fn image_format(&self) -> Option<&str> {
        match self.mime_type.as_str() {
            "image/jpeg" => Some("jpeg"),
            "image/png" => Some("png"),
            "image/gif" => Some("gif"),
            "image/bmp" => Some("bmp"),
            "image/webp" => Some("webp"),
            _ => None,
        }
    }

    /// Returns the size of the cover art in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the cover art is larger than the given size
    pub fn is_larger_than(&self, bytes: usize) -> bool {
        self.data.len() > bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format_from_extension() {
        assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("MP3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("m4b"), Some(AudioFormat::M4b));
        assert_eq!(AudioFormat::from_extension("flac"), Some(AudioFormat::Flac));
        assert_eq!(AudioFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_audio_format_from_path() {
        let path = PathBuf::from("/path/to/file.mp3");
        assert_eq!(AudioFormat::from_path(&path), Some(AudioFormat::Mp3));

        let path2 = PathBuf::from("/path/to/file.FLAC");
        assert_eq!(AudioFormat::from_path(&path2), Some(AudioFormat::Flac));

        let path3 = PathBuf::from("/path/to/file");
        assert_eq!(AudioFormat::from_path(&path3), None);
    }

    #[test]
    fn test_audio_format_extension() {
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFormat::M4b.extension(), "m4b");
        assert_eq!(AudioFormat::Flac.extension(), "flac");
    }

    #[test]
    fn test_audio_format_mime_type() {
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioFormat::M4a.mime_type(), "audio/mp4");
        assert_eq!(AudioFormat::Flac.mime_type(), "audio/flac");
    }

    #[test]
    fn test_audio_format_supports_chapters() {
        assert!(AudioFormat::Mp3.supports_chapters());
        assert!(AudioFormat::M4b.supports_chapters());
        assert!(AudioFormat::Flac.supports_chapters());
        assert!(!AudioFormat::Wav.supports_chapters());
        assert!(!AudioFormat::Aac.supports_chapters());
    }

    #[test]
    fn test_audio_format_supports_cover_art() {
        assert!(AudioFormat::Mp3.supports_cover_art());
        assert!(AudioFormat::M4a.supports_cover_art());
        assert!(AudioFormat::Flac.supports_cover_art());
        assert!(!AudioFormat::Wav.supports_cover_art());
        assert!(!AudioFormat::Aiff.supports_cover_art());
    }

    #[test]
    fn test_audio_format_display() {
        assert_eq!(AudioFormat::Mp3.to_string(), "MP3");
        assert_eq!(AudioFormat::Flac.to_string(), "FLAC");
    }

    #[test]
    fn test_audio_metadata_new() {
        let metadata = AudioMetadata::new(AudioFormat::Mp3, Duration::from_seconds(3600));

        assert_eq!(metadata.format, AudioFormat::Mp3);
        assert_eq!(metadata.duration.as_seconds(), 3600);
        assert!(metadata.title.is_none());
        assert!(!metadata.has_cover_art);
    }

    #[test]
    fn test_audio_metadata_has_technical_info() {
        let mut metadata = AudioMetadata::new(AudioFormat::Mp3, Duration::from_seconds(100));
        assert!(!metadata.has_technical_info());

        metadata.bitrate = Some(128000);
        metadata.sample_rate = Some(44100);
        metadata.channels = Some(2);
        assert!(metadata.has_technical_info());
    }

    #[test]
    fn test_audio_metadata_technical_summary() {
        let mut metadata = AudioMetadata::new(AudioFormat::Mp3, Duration::from_seconds(100));
        assert!(metadata.technical_summary().is_none());

        metadata.bitrate = Some(128000);
        metadata.sample_rate = Some(44100);
        metadata.channels = Some(2);

        let summary = metadata.technical_summary().unwrap();
        assert!(summary.contains("128 kbps"));
        assert!(summary.contains("44 kHz"));
        assert!(summary.contains("Stereo"));
    }

    #[test]
    fn test_audio_metadata_technical_summary_mono() {
        let mut metadata = AudioMetadata::new(AudioFormat::Mp3, Duration::from_seconds(100));
        metadata.bitrate = Some(64000);
        metadata.sample_rate = Some(22050);
        metadata.channels = Some(1);

        let summary = metadata.technical_summary().unwrap();
        assert!(summary.contains("Mono"));
    }

    #[test]
    fn test_cover_art_new() {
        let data = vec![0xFF, 0xD8, 0xFF]; // JPEG header
        let art = CoverArt::new(data.clone(), "image/jpeg".to_string());

        assert_eq!(art.data, data);
        assert_eq!(art.mime_type, "image/jpeg");
    }

    #[test]
    fn test_cover_art_image_format() {
        let art = CoverArt::new(vec![1, 2, 3], "image/jpeg".to_string());
        assert_eq!(art.image_format(), Some("jpeg"));

        let art2 = CoverArt::new(vec![1, 2, 3], "image/png".to_string());
        assert_eq!(art2.image_format(), Some("png"));

        let art3 = CoverArt::new(vec![1, 2, 3], "image/unknown".to_string());
        assert_eq!(art3.image_format(), None);
    }

    #[test]
    fn test_cover_art_size() {
        let art = CoverArt::new(vec![1, 2, 3, 4, 5], "image/jpeg".to_string());
        assert_eq!(art.size(), 5);
    }

    #[test]
    fn test_cover_art_is_larger_than() {
        let art = CoverArt::new(vec![1, 2, 3, 4, 5], "image/jpeg".to_string());
        assert!(art.is_larger_than(3));
        assert!(!art.is_larger_than(10));
    }

    #[test]
    fn test_all_formats_have_extensions() {
        let formats = [
            AudioFormat::Mp3,
            AudioFormat::M4b,
            AudioFormat::M4a,
            AudioFormat::Flac,
            AudioFormat::Ogg,
            AudioFormat::Opus,
            AudioFormat::Aac,
            AudioFormat::Wav,
            AudioFormat::Aiff,
        ];

        for format in &formats {
            assert!(!format.extension().is_empty());
            assert!(!format.mime_type().is_empty());
        }
    }
}
