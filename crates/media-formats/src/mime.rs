// FILE: src/mime.rs
// ============================================================================

use crate::AudioFormat;

/// MIME type information for audio formats
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeType {
    primary: &'static str,
    alternatives: Vec<&'static str>,
}

impl MimeType {
    /// Returns the MIME type for a given audio format
    pub fn from_format(format: AudioFormat) -> Self {
        match format {
            AudioFormat::Mp3 => MimeType {
                primary: "audio/mpeg",
                alternatives: vec![],
            },
            AudioFormat::M4a => MimeType {
                primary: "audio/mp4",
                alternatives: vec![],
            },
            AudioFormat::M4b => MimeType {
                primary: "audio/mp4",
                alternatives: vec![],
            },
            AudioFormat::Vorbis => MimeType {
                primary: "audio/ogg",
                alternatives: vec![],
            },
            AudioFormat::Opus => MimeType {
                primary: "audio/opus",
                alternatives: vec![],
            },
            AudioFormat::Wma => MimeType {
                primary: "audio/x-ms-wma",
                alternatives: vec![],
            },
            AudioFormat::Flac => MimeType {
                primary: "audio/flac",
                alternatives: vec![],
            },
            AudioFormat::Alac => MimeType {
                primary: "audio/x-alac",
                alternatives: vec![],
            },
            AudioFormat::Ape => MimeType {
                primary: "audio/x-ape",
                alternatives: vec![],
            },
            AudioFormat::WavPack => MimeType {
                primary: "audio/x-wavpack",
                alternatives: vec![],
            },
            AudioFormat::Tta => MimeType {
                primary: "audio/x-tta",
                alternatives: vec![],
            },
            AudioFormat::Wav => MimeType {
                primary: "audio/wav",
                alternatives: vec![],
            },
            AudioFormat::Aiff => MimeType {
                primary: "audio/aiff",
                alternatives: vec![],
            },
            AudioFormat::Mka => MimeType {
                primary: "audio/x-matroska",
                alternatives: vec![],
            },
            AudioFormat::Webm => MimeType {
                primary: "audio/webm",
                alternatives: vec![],
            },
        }
    }

    /// Returns the primary MIME type
    pub fn primary(&self) -> &str {
        self.primary
    }

    /// Returns all MIME types (primary + alternatives)
    pub fn all(&self) -> Vec<&str> {
        let mut result = vec![self.primary];
        result.extend(self.alternatives.iter().copied());
        result
    }
}

#[cfg(test)]
mod mime_tests {
    use super::*;

    #[test]
    fn test_mime_type_mp3() {
        let mime = MimeType::from_format(AudioFormat::Mp3);
        assert_eq!(mime.primary(), "audio/mpeg"); // Correct MIME type
                                                  // MP3's standard MIME type is audio/mpeg, not audio/mp3
        assert!(mime.all().contains(&"audio/mpeg"));
    }

    #[test]
    fn test_mime_type_flac() {
        let mime = MimeType::from_format(AudioFormat::Flac);
        assert_eq!(mime.primary(), "audio/flac");
        assert!(mime.all().contains(&"audio/flac"));
    }

    #[test]
    fn test_all_formats_have_mime() {
        for format in AudioFormat::all() {
            let mime = MimeType::from_format(format);
            assert!(!mime.primary().is_empty());
            assert!(!mime.all().is_empty());
        }
    }
}
