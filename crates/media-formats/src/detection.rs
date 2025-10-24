// FILE: src/detection.rs
// ============================================================================

use crate::{AudioFormat, FormatError, FormatResult};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Format detector using file content analysis
pub struct FormatDetector;

impl FormatDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detects format from file content (magic bytes)
    ///
    /// This reads the first few bytes of the file to determine format.
    /// Falls back to extension-based detection if magic bytes are inconclusive.
    pub fn detect_from_file(&self, path: &Path) -> FormatResult<AudioFormat> {
        // Try magic bytes first
        if let Ok(format) = self.detect_from_magic_bytes(path) {
            return Ok(format);
        }

        // Fall back to extension
        AudioFormat::from_path(path).ok_or(FormatError::UnknownFormat)
    }

    /// Detects format from magic bytes
    fn detect_from_magic_bytes(&self, path: &Path) -> FormatResult<AudioFormat> {
        let mut file = File::open(path).map_err(|e| FormatError::IoError(e.to_string()))?;

        let mut buffer = [0u8; 16];
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| FormatError::IoError(e.to_string()))?;

        if bytes_read < 4 {
            return Err(FormatError::InvalidMagicBytes);
        }

        // Check magic bytes
        if buffer.starts_with(b"ID3")
            || buffer[0..2] == [0xFF, 0xFB]
            || buffer[0..2] == [0xFF, 0xFA]
        {
            return Ok(AudioFormat::Mp3);
        }

        if buffer.starts_with(b"fLaC") {
            return Ok(AudioFormat::Flac);
        }

        if buffer.starts_with(b"OggS") {
            return Ok(AudioFormat::Opus);
        }

        if buffer.starts_with(b"RIFF") && bytes_read >= 12 && &buffer[8..12] == b"WAVE" {
            return Ok(AudioFormat::Wav);
        }

        if buffer.starts_with(b"FORM") && bytes_read >= 12 && &buffer[8..12] == b"AIFF" {
            return Ok(AudioFormat::Aiff);
        }

        if buffer[4..8] == *b"ftyp" {
            // MPEG-4 container (M4A/M4B)
            return Ok(AudioFormat::M4a);
        }

        if buffer.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
            // Matroska/WebM
            return Ok(AudioFormat::Mka);
        }

        if buffer.starts_with(b"MAC ") {
            return Ok(AudioFormat::Ape);
        }

        if buffer.starts_with(b"wvpk") {
            return Ok(AudioFormat::WavPack);
        }

        if buffer.starts_with(b"TTA1") {
            return Ok(AudioFormat::Tta);
        }

        Err(FormatError::InvalidMagicBytes)
    }
}

impl Default for FormatDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod detection_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file_with_content(content: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content).expect("Failed to write content");
        file.flush().expect("Failed to flush");
        file
    }

    #[test]
    fn test_format_detector_creation() {
        let detector = FormatDetector::new();
        // FormatDetector is a unit struct (zero-sized type)
        // Just verify it can be created and used
        let _ = detector;
        // Or test that it actually works with a real file
        let file = create_temp_file_with_content(b"ID3\x03\x00\x00\x00\x00\x00\x00");
        assert!(detector.detect_from_file(file.path()).is_ok());
    }

    #[test]
    fn test_detect_mp3_id3() {
        let detector = FormatDetector::new();
        let file = create_temp_file_with_content(b"ID3\x03\x00\x00\x00\x00\x00\x00");
        let result = detector.detect_from_magic_bytes(file.path());
        assert_eq!(result, Ok(AudioFormat::Mp3));
    }

    #[test]
    fn test_detect_flac() {
        let detector = FormatDetector::new();
        let file = create_temp_file_with_content(b"fLaC\x00\x00\x00\x22");
        let result = detector.detect_from_magic_bytes(file.path());
        assert_eq!(result, Ok(AudioFormat::Flac));
    }

    #[test]
    fn test_detect_ogg() {
        let detector = FormatDetector::new();
        let file = create_temp_file_with_content(b"OggS\x00\x02\x00\x00");
        let result = detector.detect_from_magic_bytes(file.path());
        assert_eq!(result, Ok(AudioFormat::Opus));
    }

    #[test]
    fn test_detect_wav() {
        let detector = FormatDetector::new();
        let content = b"RIFF\x00\x00\x00\x00WAVEfmt ";
        let file = create_temp_file_with_content(content);
        let result = detector.detect_from_magic_bytes(file.path());
        assert_eq!(result, Ok(AudioFormat::Wav));
    }

    #[test]
    fn test_detect_invalid_magic_bytes() {
        let detector = FormatDetector::new();
        let file = create_temp_file_with_content(b"INVALID\x00\x00\x00");
        let result = detector.detect_from_magic_bytes(file.path());
        assert_eq!(result, Err(FormatError::InvalidMagicBytes));
    }

    #[test]
    fn test_detect_too_short() {
        let detector = FormatDetector::new();
        let file = create_temp_file_with_content(b"ID");
        let result = detector.detect_from_magic_bytes(file.path());
        assert_eq!(result, Err(FormatError::InvalidMagicBytes));
    }
}
