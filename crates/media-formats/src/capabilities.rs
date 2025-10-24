// FILE: src/capabilities.rs
// ============================================================================

use crate::AudioFormat;

/// Format capabilities and features
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatCapabilities {
    /// Format supports embedded metadata (ID3, Vorbis comments, etc.)
    pub metadata: MetadataSupport,
    /// Format supports embedded cover art
    pub cover_art: bool,
    /// Format supports chapter markers
    pub chapters: bool,
    /// Format supports streaming
    pub streaming: bool,
    /// Format is seekable
    pub seekable: bool,
    /// Typical quality level
    pub quality: QualityLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataSupport {
    None,
    Basic,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLevel {
    Lossy,
    Lossless,
    Uncompressed,
}

impl FormatCapabilities {
    /// Returns capabilities for a given format
    pub fn for_format(format: AudioFormat) -> Self {
        match format {
            AudioFormat::Mp3 => Self {
                metadata: MetadataSupport::Full,
                cover_art: true,
                chapters: true,
                streaming: true,
                seekable: true,
                quality: QualityLevel::Lossy,
            },
            AudioFormat::M4b => Self {
                metadata: MetadataSupport::Full,
                cover_art: true,
                chapters: true,
                streaming: true,
                seekable: true,
                quality: QualityLevel::Lossy,
            },
            AudioFormat::M4a => Self {
                metadata: MetadataSupport::Full,
                cover_art: true,
                chapters: false,
                streaming: true,
                seekable: true,
                quality: QualityLevel::Lossy,
            },
            AudioFormat::Flac => Self {
                metadata: MetadataSupport::Full,
                cover_art: true,
                chapters: true,
                streaming: true,
                seekable: true,
                quality: QualityLevel::Lossless,
            },
            AudioFormat::Opus => Self {
                metadata: MetadataSupport::Full,
                cover_art: true,
                chapters: false,
                streaming: true,
                seekable: true,
                quality: QualityLevel::Lossy,
            },
            AudioFormat::Wav => Self {
                metadata: MetadataSupport::Basic,
                cover_art: false,
                chapters: false,
                streaming: true,
                seekable: true,
                quality: QualityLevel::Uncompressed,
            },
            AudioFormat::Aiff => Self {
                metadata: MetadataSupport::Basic,
                cover_art: false,
                chapters: false,
                streaming: true,
                seekable: true,
                quality: QualityLevel::Uncompressed,
            },
            _ => Self::default(),
        }
    }
}

impl Default for FormatCapabilities {
    fn default() -> Self {
        Self {
            metadata: MetadataSupport::Basic,
            cover_art: false,
            chapters: false,
            streaming: true,
            seekable: true,
            quality: QualityLevel::Lossy,
        }
    }
}

#[cfg(test)]
mod capabilities_tests {
    use super::*;

    #[test]
    fn test_mp3_capabilities() {
        let caps = FormatCapabilities::for_format(AudioFormat::Mp3);
        assert_eq!(caps.metadata, MetadataSupport::Full);
        assert!(caps.cover_art);
        assert!(caps.chapters);
    }

    #[test]
    fn test_wav_capabilities() {
        let caps = FormatCapabilities::for_format(AudioFormat::Wav);
        assert_eq!(caps.metadata, MetadataSupport::Basic);
        assert!(!caps.cover_art);
        assert!(!caps.chapters);
    }

    #[test]
    fn test_all_formats_have_capabilities() {
        for format in AudioFormat::all() {
            let caps = FormatCapabilities::for_format(format);
            assert!(caps.seekable); // All formats should be seekable
        }
    }
}
