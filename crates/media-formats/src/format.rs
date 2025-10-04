// FILE: src/format.rs
// ============================================================================

use core::fmt;
use std::path::Path;

/// Supported audio formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioFormat {
    // Lossy formats
    Mp3,
    Aac,
    M4a,
    M4b,
    Ogg,
    Opus,
    Wma,

    // Lossless formats
    Flac,
    Alac,
    Ape,
    WavPack,
    Tta,

    // Uncompressed formats
    Wav,
    Aiff,

    // Container formats
    Mka,
    Webm,
}

impl AudioFormat {
    /// Returns all supported formats
    pub fn all() -> Vec<AudioFormat> {
        vec![
            AudioFormat::Mp3,
            AudioFormat::Aac,
            AudioFormat::M4a,
            AudioFormat::M4b,
            AudioFormat::Ogg,
            AudioFormat::Opus,
            AudioFormat::Wma,
            AudioFormat::Flac,
            AudioFormat::Alac,
            AudioFormat::Ape,
            AudioFormat::WavPack,
            AudioFormat::Tta,
            AudioFormat::Wav,
            AudioFormat::Aiff,
            AudioFormat::Mka,
            AudioFormat::Webm,
        ]
    }

    /// Detects format from file extension
    ///
    /// # Examples
    ///
    /// ```
    /// # use storystream_media_formats::AudioFormat;
    /// assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
    /// assert_eq!(AudioFormat::from_extension("MP3"), Some(AudioFormat::Mp3));
    /// assert_eq!(AudioFormat::from_extension(".flac"), Some(AudioFormat::Flac));
    /// assert_eq!(AudioFormat::from_extension("unknown"), None);
    /// ```
    pub fn from_extension(ext: &str) -> Option<AudioFormat> {
        let ext = ext.trim_start_matches('.').to_lowercase();

        match ext.as_str() {
            "mp3" => Some(AudioFormat::Mp3),
            "aac" => Some(AudioFormat::Aac),
            "m4a" => Some(AudioFormat::M4a),
            "m4b" => Some(AudioFormat::M4b),
            "ogg" | "oga" => Some(AudioFormat::Ogg),
            "opus" => Some(AudioFormat::Opus),
            "wma" => Some(AudioFormat::Wma),
            "flac" => Some(AudioFormat::Flac),
            "alac" => Some(AudioFormat::Alac),
            "ape" => Some(AudioFormat::Ape),
            "wv" => Some(AudioFormat::WavPack),
            "tta" => Some(AudioFormat::Tta),
            "wav" | "wave" => Some(AudioFormat::Wav),
            "aiff" | "aif" | "aifc" => Some(AudioFormat::Aiff),
            "mka" => Some(AudioFormat::Mka),
            "webm" => Some(AudioFormat::Webm),
            _ => None,
        }
    }

    /// Detects format from file path
    ///
    /// # Examples
    ///
    /// ```
    /// # use storystream_media_formats::AudioFormat;
    /// # use std::path::Path;
    /// let path = Path::new("audiobook.mp3");
    /// assert_eq!(AudioFormat::from_path(path), Some(AudioFormat::Mp3));
    /// ```
    pub fn from_path(path: &Path) -> Option<AudioFormat> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(AudioFormat::from_extension)
    }

    /// Returns the primary file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Aac => "aac",
            AudioFormat::M4a => "m4a",
            AudioFormat::M4b => "m4b",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Opus => "opus",
            AudioFormat::Wma => "wma",
            AudioFormat::Flac => "flac",
            AudioFormat::Alac => "alac",
            AudioFormat::Ape => "ape",
            AudioFormat::WavPack => "wv",
            AudioFormat::Tta => "tta",
            AudioFormat::Wav => "wav",
            AudioFormat::Aiff => "aiff",
            AudioFormat::Mka => "mka",
            AudioFormat::Webm => "webm",
        }
    }

    /// Returns all valid extensions for this format
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            AudioFormat::Mp3 => vec!["mp3"],
            AudioFormat::Aac => vec!["aac"],
            AudioFormat::M4a => vec!["m4a"],
            AudioFormat::M4b => vec!["m4b"],
            AudioFormat::Ogg => vec!["ogg", "oga"],
            AudioFormat::Opus => vec!["opus"],
            AudioFormat::Wma => vec!["wma"],
            AudioFormat::Flac => vec!["flac"],
            AudioFormat::Alac => vec!["alac"],
            AudioFormat::Ape => vec!["ape"],
            AudioFormat::WavPack => vec!["wv"],
            AudioFormat::Tta => vec!["tta"],
            AudioFormat::Wav => vec!["wav", "wave"],
            AudioFormat::Aiff => vec!["aiff", "aif", "aifc"],
            AudioFormat::Mka => vec!["mka"],
            AudioFormat::Webm => vec!["webm"],
        }
    }

    /// Returns the format name
    pub fn name(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "MP3",
            AudioFormat::Aac => "AAC",
            AudioFormat::M4a => "M4A",
            AudioFormat::M4b => "M4B Audiobook",
            AudioFormat::Ogg => "Ogg Vorbis",
            AudioFormat::Opus => "Opus",
            AudioFormat::Wma => "Windows Media Audio",
            AudioFormat::Flac => "FLAC",
            AudioFormat::Alac => "Apple Lossless",
            AudioFormat::Ape => "Monkey's Audio",
            AudioFormat::WavPack => "WavPack",
            AudioFormat::Tta => "True Audio",
            AudioFormat::Wav => "WAV",
            AudioFormat::Aiff => "AIFF",
            AudioFormat::Mka => "Matroska Audio",
            AudioFormat::Webm => "WebM Audio",
        }
    }

    /// Returns whether this is a lossy format
    pub fn is_lossy(&self) -> bool {
        matches!(
            self,
            AudioFormat::Mp3
                | AudioFormat::Aac
                | AudioFormat::M4a
                | AudioFormat::M4b
                | AudioFormat::Ogg
                | AudioFormat::Opus
                | AudioFormat::Wma
        )
    }

    /// Returns whether this is a lossless format
    pub fn is_lossless(&self) -> bool {
        matches!(
            self,
            AudioFormat::Flac
                | AudioFormat::Alac
                | AudioFormat::Ape
                | AudioFormat::WavPack
                | AudioFormat::Tta
        )
    }

    /// Returns whether this is an uncompressed format
    pub fn is_uncompressed(&self) -> bool {
        matches!(self, AudioFormat::Wav | AudioFormat::Aiff)
    }

    /// Returns whether this format supports embedded metadata
    pub fn supports_metadata(&self) -> bool {
        !matches!(self, AudioFormat::Wav | AudioFormat::Aiff)
    }

    /// Returns whether this format supports cover art
    pub fn supports_cover_art(&self) -> bool {
        matches!(
            self,
            AudioFormat::Mp3
                | AudioFormat::M4a
                | AudioFormat::M4b
                | AudioFormat::Flac
                | AudioFormat::Ogg
                | AudioFormat::Opus
                | AudioFormat::Mka
                | AudioFormat::WavPack
        )
    }

    /// Returns whether this format supports chapters
    pub fn supports_chapters(&self) -> bool {
        matches!(
            self,
            AudioFormat::M4b | AudioFormat::Mp3 | AudioFormat::Mka | AudioFormat::Flac
        )
    }

    /// Returns whether this format is commonly used for audiobooks
    pub fn is_audiobook_format(&self) -> bool {
        matches!(
            self,
            AudioFormat::M4b | AudioFormat::Mp3 | AudioFormat::M4a | AudioFormat::Opus
        )
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;

    #[test]
    fn test_all_formats_count() {
        assert_eq!(AudioFormat::all().len(), 16);
    }

    #[test]
    fn test_from_extension_case_insensitive() {
        assert_eq!(AudioFormat::from_extension("MP3"), Some(AudioFormat::Mp3));
        assert_eq!(
            AudioFormat::from_extension("FLAC"),
            Some(AudioFormat::Flac)
        );
        assert_eq!(AudioFormat::from_extension("m4b"), Some(AudioFormat::M4b));
    }

    #[test]
    fn test_from_extension_with_dot() {
        assert_eq!(
            AudioFormat::from_extension(".mp3"),
            Some(AudioFormat::Mp3)
        );
        assert_eq!(
            AudioFormat::from_extension(".flac"),
            Some(AudioFormat::Flac)
        );
    }

    #[test]
    fn test_from_extension_unknown() {
        assert_eq!(AudioFormat::from_extension("xyz"), None);
        assert_eq!(AudioFormat::from_extension(""), None);
    }

    #[test]
    fn test_from_path() {
        let path = Path::new("test.mp3");
        assert_eq!(AudioFormat::from_path(path), Some(AudioFormat::Mp3));

        let path = Path::new("/path/to/audiobook.m4b");
        assert_eq!(AudioFormat::from_path(path), Some(AudioFormat::M4b));
    }

    #[test]
    fn test_extension() {
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFormat::Flac.extension(), "flac");
        assert_eq!(AudioFormat::M4b.extension(), "m4b");
    }

    #[test]
    fn test_extensions() {
        assert_eq!(AudioFormat::Mp3.extensions(), vec!["mp3"]);
        assert!(AudioFormat::Ogg.extensions().contains(&"ogg"));
        assert!(AudioFormat::Ogg.extensions().contains(&"oga"));
    }

    #[test]
    fn test_name() {
        assert_eq!(AudioFormat::Mp3.name(), "MP3");
        assert_eq!(AudioFormat::M4b.name(), "M4B Audiobook");
        assert_eq!(AudioFormat::Flac.name(), "FLAC");
    }

    #[test]
    fn test_lossy_formats() {
        assert!(AudioFormat::Mp3.is_lossy());
        assert!(AudioFormat::Aac.is_lossy());
        assert!(AudioFormat::Ogg.is_lossy());
        assert!(!AudioFormat::Flac.is_lossy());
        assert!(!AudioFormat::Wav.is_lossy());
    }

    #[test]
    fn test_lossless_formats() {
        assert!(AudioFormat::Flac.is_lossless());
        assert!(AudioFormat::Alac.is_lossless());
        assert!(AudioFormat::WavPack.is_lossless());
        assert!(!AudioFormat::Mp3.is_lossless());
    }

    #[test]
    fn test_uncompressed_formats() {
        assert!(AudioFormat::Wav.is_uncompressed());
        assert!(AudioFormat::Aiff.is_uncompressed());
        assert!(!AudioFormat::Flac.is_uncompressed());
        assert!(!AudioFormat::Mp3.is_uncompressed());
    }

    #[test]
    fn test_supports_metadata() {
        assert!(AudioFormat::Mp3.supports_metadata());
        assert!(AudioFormat::Flac.supports_metadata());
        assert!(!AudioFormat::Wav.supports_metadata());
    }

    #[test]
    fn test_supports_cover_art() {
        assert!(AudioFormat::Mp3.supports_cover_art());
        assert!(AudioFormat::M4b.supports_cover_art());
        assert!(AudioFormat::Flac.supports_cover_art());
        assert!(!AudioFormat::Wav.supports_cover_art());
        assert!(!AudioFormat::Aac.supports_cover_art());
    }

    #[test]
    fn test_supports_chapters() {
        assert!(AudioFormat::M4b.supports_chapters());
        assert!(AudioFormat::Mp3.supports_chapters());
        assert!(!AudioFormat::Aac.supports_chapters());
    }

    #[test]
    fn test_audiobook_formats() {
        assert!(AudioFormat::M4b.is_audiobook_format());
        assert!(AudioFormat::Mp3.is_audiobook_format());
        assert!(!AudioFormat::Wav.is_audiobook_format());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", AudioFormat::Mp3), "MP3");
        assert_eq!(format!("{}", AudioFormat::M4b), "M4B Audiobook");
    }

    #[test]
    fn test_all_formats_have_extension() {
        for format in AudioFormat::all() {
            assert!(!format.extension().is_empty());
        }
    }

    #[test]
    fn test_all_formats_have_name() {
        for format in AudioFormat::all() {
            assert!(!format.name().is_empty());
        }
    }
}