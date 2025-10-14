//! Audio format types and capabilities

use std::fmt;
use std::path::Path;

/// Supported audio formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioFormat {
    // === Lossless Formats ===
    /// FLAC - Free Lossless Audio Codec
    Flac,
    /// ALAC - Apple Lossless Audio Codec
    Alac,
    /// APE - Monkey's Audio
    Ape,
    /// WavPack - Hybrid lossless compression
    WavPack,
    /// TTA - True Audio
    Tta,

    // === Uncompressed Formats ===
    /// WAV - Waveform Audio File Format
    Wav,
    /// AIFF - Audio Interchange File Format
    Aiff,

    // === High-Quality Lossy Formats ===
    /// Opus - Modern, highly efficient codec
    Opus,
    /// Vorbis - OGG Vorbis
    Vorbis,
    /// AAC - Advanced Audio Coding (M4A container)
    M4a,
    /// M4B - AAC in M4B container (audiobook format with chapters)
    M4b,

    // === Legacy Lossy Formats ===
    /// MP3 - MPEG Audio Layer 3
    Mp3,
    /// WMA - Windows Media Audio
    Wma,

    // === Container Formats ===
    /// MKA - Matroska Audio
    Mka,
    /// WebM Audio
    Webm,
}

impl AudioFormat {
    /// Returns all supported formats
    pub fn all() -> Vec<Self> {
        vec![
            Self::Flac,
            Self::Alac,
            Self::Ape,
            Self::WavPack,
            Self::Tta,
            Self::Wav,
            Self::Aiff,
            Self::Opus,
            Self::Vorbis,
            Self::M4a,
            Self::M4b,
            Self::Mp3,
            Self::Wma,
            Self::Mka,
            Self::Webm,
        ]
    }

    /// Detects format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext = ext.trim_start_matches('.').to_lowercase();
        match ext.as_str() {
            "flac" => Some(Self::Flac),
            "alac" => Some(Self::Alac),
            "ape" => Some(Self::Ape),
            "wv" => Some(Self::WavPack),
            "tta" => Some(Self::Tta),
            "wav" | "wave" => Some(Self::Wav),
            "aiff" | "aif" | "aifc" => Some(Self::Aiff),
            "opus" => Some(Self::Opus),
            "ogg" | "oga" => Some(Self::Vorbis),
            "m4a" => Some(Self::M4a),
            "m4b" => Some(Self::M4b),
            "mp3" => Some(Self::Mp3),
            "wma" => Some(Self::Wma),
            "mka" => Some(Self::Mka),
            "webm" => Some(Self::Webm),
            _ => None,
        }
    }

    /// Detects format from file path
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Returns the canonical file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Flac => "flac",
            Self::Alac => "alac",
            Self::Ape => "ape",
            Self::WavPack => "wv",
            Self::Tta => "tta",
            Self::Wav => "wav",
            Self::Aiff => "aiff",
            Self::Opus => "opus",
            Self::Vorbis => "ogg",
            Self::M4a => "m4a",
            Self::M4b => "m4b",
            Self::Mp3 => "mp3",
            Self::Wma => "wma",
            Self::Mka => "mka",
            Self::Webm => "webm",
        }
    }

    /// Returns the format name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Flac => "FLAC",
            Self::Alac => "ALAC",
            Self::Ape => "APE",
            Self::WavPack => "WavPack",
            Self::Tta => "TTA",
            Self::Wav => "WAV",
            Self::Aiff => "AIFF",
            Self::Opus => "Opus",
            Self::Vorbis => "Vorbis",
            Self::M4a => "AAC (M4A)",
            Self::M4b => "AAC (M4B)",
            Self::Mp3 => "MP3",
            Self::Wma => "WMA",
            Self::Mka => "Matroska Audio",
            Self::Webm => "WebM Audio",
        }
    }

    /// Returns the MIME type
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Flac => "audio/flac",
            Self::Alac => "audio/x-alac",
            Self::Ape => "audio/x-ape",
            Self::WavPack => "audio/x-wavpack",
            Self::Tta => "audio/x-tta",
            Self::Wav => "audio/wav",
            Self::Aiff => "audio/aiff",
            Self::Opus => "audio/opus",
            Self::Vorbis => "audio/ogg",
            Self::M4a | Self::M4b => "audio/mp4",
            Self::Mp3 => "audio/mpeg",
            Self::Wma => "audio/x-ms-wma",
            Self::Mka => "audio/x-matroska",
            Self::Webm => "audio/webm",
        }
    }

    /// Returns true if this is a lossy format
    pub fn is_lossy(&self) -> bool {
        matches!(
            self,
            Self::Opus | Self::Vorbis | Self::M4a | Self::M4b | Self::Mp3 | Self::Wma
        )
    }

    /// Returns true if this is a lossless format
    pub fn is_lossless(&self) -> bool {
        matches!(
            self,
            Self::Flac | Self::Alac | Self::Ape | Self::WavPack | Self::Tta
        )
    }

    /// Returns true if this is uncompressed
    pub fn is_uncompressed(&self) -> bool {
        matches!(self, Self::Wav | Self::Aiff)
    }

    /// Returns true if this format supports embedded metadata
    pub fn supports_metadata(&self) -> bool {
        !self.is_uncompressed()
    }

    /// Returns true if this format supports cover art
    pub fn supports_cover_art(&self) -> bool {
        matches!(
            self,
            Self::Flac
                | Self::Alac
                | Self::M4a
                | Self::M4b
                | Self::Mp3
                | Self::Vorbis
                | Self::Opus
                | Self::WavPack
                | Self::Mka
        )
    }

    /// Returns true if this format supports chapter markers
    pub fn supports_chapters(&self) -> bool {
        matches!(
            self,
            Self::M4b | Self::Mp3 | Self::Flac | Self::Alac | Self::Mka
        )
    }

    /// Returns true if this format is commonly used for audiobooks
    pub fn is_audiobook_format(&self) -> bool {
        matches!(self, Self::M4b | Self::Mp3 | Self::M4a | Self::Opus)
    }

    /// Returns format capabilities
    pub fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities::for_format(*self)
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Format capabilities
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatCapabilities {
    /// Supports embedded metadata
    pub metadata: bool,
    /// Supports embedded cover art
    pub cover_art: bool,
    /// Supports chapter markers
    pub chapters: bool,
    /// Supports streaming (can start playback before full download)
    pub streaming: bool,
    /// Seekable
    pub seekable: bool,
    /// Lossless compression
    pub lossless: bool,
    /// Variable bitrate support
    pub variable_bitrate: bool,
}

impl FormatCapabilities {
    /// Returns capabilities for a format
    pub fn for_format(format: AudioFormat) -> Self {
        Self {
            metadata: format.supports_metadata(),
            cover_art: format.supports_cover_art(),
            chapters: format.supports_chapters(),
            streaming: !matches!(format, AudioFormat::Ape | AudioFormat::Tta),
            seekable: true,
            lossless: format.is_lossless() || format.is_uncompressed(),
            variable_bitrate: !format.is_uncompressed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_formats_count() {
        assert_eq!(AudioFormat::all().len(), 15);
    }

    #[test]
    fn test_extension_detection() {
        assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("MP3"), Some(AudioFormat::Mp3));
        assert_eq!(
            AudioFormat::from_extension(".flac"),
            Some(AudioFormat::Flac)
        );
        assert_eq!(AudioFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_format_properties() {
        assert!(AudioFormat::Flac.is_lossless());
        assert!(AudioFormat::Mp3.is_lossy());
        assert!(AudioFormat::Wav.is_uncompressed());
        assert!(!AudioFormat::Mp3.is_lossless());
    }

    #[test]
    fn test_chapter_support() {
        assert!(AudioFormat::M4b.supports_chapters());
        assert!(AudioFormat::Flac.supports_chapters());
        assert!(!AudioFormat::Wav.supports_chapters());
    }

    #[test]
    fn test_audiobook_formats() {
        assert!(AudioFormat::M4b.is_audiobook_format());
        assert!(AudioFormat::Mp3.is_audiobook_format());
        assert!(!AudioFormat::Wav.is_audiobook_format());
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioFormat::Flac.mime_type(), "audio/flac");
        assert_eq!(AudioFormat::M4b.mime_type(), "audio/mp4");
    }

    #[test]
    fn test_capabilities() {
        let caps = AudioFormat::Flac.capabilities();
        assert!(caps.metadata);
        assert!(caps.cover_art);
        assert!(caps.chapters);
        assert!(caps.lossless);

        let caps = AudioFormat::Wav.capabilities();
        assert!(!caps.metadata);
        assert!(!caps.cover_art);
        assert!(caps.lossless);
    }
}