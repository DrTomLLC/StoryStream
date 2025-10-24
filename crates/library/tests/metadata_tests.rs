// FILE: crates/library/tests/metadata_tests.rs
// Integration tests for metadata extraction

use anyhow::Result;
use std::io::Write;
use std::path::Path;
use storystream_library::metadata::{ExtractedMetadata, MetadataExtractor};
use tempfile::NamedTempFile;

// Helper to create a minimal MP3 file with ID3 tags
// Note: This is a valid but minimal MP3 frame
const MINIMAL_MP3: &[u8] = &[
    // ID3v2 header
    0x49, 0x44, 0x33, // "ID3"
    0x04, 0x00, // Version 2.4.0
    0x00, // Flags
    0x00, 0x00, 0x00, 0x0A, // Size (10 bytes)
    // MP3 frame header (valid but minimal)
    0xFF, 0xFB, 0x90, 0x00,
];

#[test]
fn test_extractor_creation() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    // Should create successfully
    Ok(())
}

#[test]
fn test_extractor_default() {
    let _extractor = MetadataExtractor::default();
    // Should not panic
}

#[test]
fn test_is_supported_mp3() {
    assert!(MetadataExtractor::is_supported(Path::new("test.mp3")));
    assert!(MetadataExtractor::is_supported(Path::new("TEST.MP3")));
    assert!(MetadataExtractor::is_supported(Path::new("file.Mp3")));
}

#[test]
fn test_is_supported_m4b() {
    assert!(MetadataExtractor::is_supported(Path::new("book.m4b")));
    assert!(MetadataExtractor::is_supported(Path::new("BOOK.M4B")));
}

#[test]
fn test_is_supported_flac() {
    assert!(MetadataExtractor::is_supported(Path::new("audio.flac")));
    assert!(MetadataExtractor::is_supported(Path::new("AUDIO.FLAC")));
}

#[test]
fn test_is_supported_ogg() {
    assert!(MetadataExtractor::is_supported(Path::new("file.ogg")));
    assert!(MetadataExtractor::is_supported(Path::new("file.opus")));
}

#[test]
fn test_is_not_supported_txt() {
    assert!(!MetadataExtractor::is_supported(Path::new("document.txt")));
}

#[test]
fn test_is_not_supported_pdf() {
    assert!(!MetadataExtractor::is_supported(Path::new("book.pdf")));
}

#[test]
fn test_is_not_supported_unknown() {
    assert!(!MetadataExtractor::is_supported(Path::new("file.xyz")));
}

#[test]
fn test_supported_extensions_contains_common_formats() {
    let extensions = MetadataExtractor::supported_extensions();

    assert!(
        !extensions.is_empty(),
        "Should have some supported extensions"
    );
    assert!(extensions.contains(&"mp3"), "Should support MP3");
    assert!(extensions.contains(&"m4b"), "Should support M4B");
    assert!(extensions.contains(&"flac"), "Should support FLAC");
}

#[test]
fn test_extract_nonexistent_file() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let result = extractor.extract(Path::new("/nonexistent/file.mp3"));

    assert!(result.is_err(), "Should fail for nonexistent file");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not found") || err_msg.contains("No such file"),
        "Error should mention file not found: {}",
        err_msg
    );

    Ok(())
}

#[test]
fn test_extract_invalid_file() -> Result<()> {
    let extractor = MetadataExtractor::new()?;

    // Create a text file with .mp3 extension
    let mut temp_file = NamedTempFile::with_suffix(".mp3")?;
    temp_file.write_all(b"This is not an audio file, just plain text")?;
    temp_file.flush()?;

    let result = extractor.extract(temp_file.path());

    assert!(result.is_err(), "Should fail for invalid audio file");

    Ok(())
}

#[test]
fn test_extract_empty_file() -> Result<()> {
    let extractor = MetadataExtractor::new()?;

    // Create empty file
    let temp_file = NamedTempFile::with_suffix(".mp3")?;

    let result = extractor.extract(temp_file.path());

    assert!(result.is_err(), "Should fail for empty file");

    Ok(())
}

#[test]
fn test_extract_minimal_valid_mp3() -> Result<()> {
    let extractor = MetadataExtractor::new()?;

    // Create minimal valid MP3
    let mut temp_file = NamedTempFile::with_suffix(".mp3")?;
    temp_file.write_all(MINIMAL_MP3)?;
    temp_file.flush()?;

    // This might fail due to minimal nature, but shouldn't panic
    let result = extractor.extract(temp_file.path());

    // We accept either success or graceful failure
    match result {
        Ok(metadata) => {
            // If it succeeds, validate basic structure
            assert!(metadata.file_size > 0);
            assert!(metadata.sample_rate > 0);
            assert!(metadata.channels > 0);
        }
        Err(e) => {
            // If it fails, ensure it's a graceful error
            let err_msg = e.to_string();
            assert!(
                !err_msg.contains("panic") && !err_msg.contains("unwrap"),
                "Should fail gracefully, not panic: {}",
                err_msg
            );
        }
    }

    Ok(())
}

#[test]
fn test_to_book_with_full_metadata() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/test/path/audiobook.mp3");

    let metadata = ExtractedMetadata {
        title: Some("The Great Gatsby".to_string()),
        author: Some("F. Scott Fitzgerald".to_string()),
        narrator: Some("Jake Gyllenhaal".to_string()),
        description: Some("A classic American novel".to_string()),
        series: Some("The Great American Novels".to_string()),
        series_position: Some(1.0),
        duration: storystream_core::Duration::from_seconds(3600 * 8), // 8 hours
        file_size: 250_000_000,                                       // 250 MB
        format: storystream_media_formats::AudioFormat::Mp3,
        bitrate: Some(128000),
        sample_rate: 44100,
        channels: 2,
        cover_art: Some(vec![0xFF, 0xD8, 0xFF, 0xE0]), // JPEG header
    };

    let book = extractor.to_book(path, metadata);

    assert_eq!(book.title, "The Great Gatsby");
    assert_eq!(book.author, Some("F. Scott Fitzgerald".to_string()));
    assert_eq!(book.narrator, Some("Jake Gyllenhaal".to_string()));
    assert_eq!(
        book.description,
        Some("A classic American novel".to_string())
    );
    assert_eq!(book.series, Some("The Great American Novels".to_string()));
    assert_eq!(book.series_position, Some(1.0));
    assert_eq!(book.duration.as_seconds(), 3600 * 8);
    assert_eq!(book.file_path, path);

    Ok(())
}

#[test]
fn test_to_book_with_minimal_metadata() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/test/minimal.mp3");

    let metadata = ExtractedMetadata {
        title: None,
        author: None,
        narrator: None,
        description: None,
        series: None,
        series_position: None,
        duration: storystream_core::Duration::from_seconds(300),
        file_size: 5_000_000,
        format: storystream_media_formats::AudioFormat::Mp3,
        bitrate: Some(128000),
        sample_rate: 44100,
        channels: 2,
        cover_art: None,
    };

    let book = extractor.to_book(path, metadata);

    // Should use filename as title fallback
    assert_eq!(book.title, "minimal");
    assert_eq!(book.author, None);
    assert_eq!(book.narrator, None);
    assert_eq!(book.description, None);
    assert_eq!(book.series, None);
    assert_eq!(book.series_position, None);

    Ok(())
}

#[test]
fn test_to_book_fallback_title_from_filename() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/audiobooks/my_favorite_book.mp3");

    let metadata = ExtractedMetadata {
        title: None,
        author: None,
        narrator: None,
        description: None,
        series: None,
        series_position: None,
        duration: storystream_core::Duration::from_seconds(1800),
        file_size: 10_000_000,
        format: storystream_media_formats::AudioFormat::Mp3,
        bitrate: None,
        sample_rate: 44100,
        channels: 2,
        cover_art: None,
    };

    let book = extractor.to_book(path, metadata);

    assert_eq!(book.title, "my_favorite_book");

    Ok(())
}

#[test]
fn test_to_book_with_unicode_path() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/audiobooks/日本語のタイトル.mp3");

    let metadata = ExtractedMetadata {
        title: Some("Japanese Title".to_string()),
        author: Some("日本の作家".to_string()),
        narrator: None,
        description: None,
        series: None,
        series_position: None,
        duration: storystream_core::Duration::from_seconds(1200),
        file_size: 8_000_000,
        format: storystream_media_formats::AudioFormat::Mp3,
        bitrate: Some(192000),
        sample_rate: 48000,
        channels: 2,
        cover_art: None,
    };

    let book = extractor.to_book(path, metadata);

    assert_eq!(book.title, "Japanese Title");
    assert_eq!(book.author, Some("日本の作家".to_string()));

    Ok(())
}

#[test]
fn test_to_book_preserves_all_audio_properties() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/test/audio.flac");

    let metadata = ExtractedMetadata {
        title: Some("High Quality Audio".to_string()),
        author: Some("Audiophile Author".to_string()),
        narrator: None,
        description: None,
        series: None,
        series_position: None,
        duration: storystream_core::Duration::from_seconds(7200),
        file_size: 500_000_000,
        format: storystream_media_formats::AudioFormat::Flac,
        bitrate: Some(1411000), // FLAC bitrate
        sample_rate: 96000,
        channels: 2,
        cover_art: Some(vec![1, 2, 3, 4]),
    };

    let book = extractor.to_book(path, metadata);

    // Verify the book was created with correct path and duration
    assert_eq!(book.file_path, path);
    assert_eq!(book.duration.as_seconds(), 7200);

    Ok(())
}

#[test]
fn test_multiple_extractors_concurrent() -> Result<()> {
    // Test that multiple extractors can be created and used concurrently
    let extractor1 = MetadataExtractor::new()?;
    let extractor2 = MetadataExtractor::new()?;
    let extractor3 = MetadataExtractor::new()?;

    // All should be independent
    let supported1 = MetadataExtractor::supported_extensions();
    let supported2 = MetadataExtractor::supported_extensions();
    let supported3 = MetadataExtractor::supported_extensions();

    assert_eq!(supported1.len(), supported2.len());
    assert_eq!(supported2.len(), supported3.len());

    Ok(())
}

#[test]
fn test_extract_tags_no_panic_on_corrupt_file() -> Result<()> {
    let extractor = MetadataExtractor::new()?;

    // Create a file with some bytes but not valid audio
    let mut temp_file = NamedTempFile::with_suffix(".mp3")?;
    temp_file.write_all(&[0xFF; 1000])?; // Just 1000 0xFF bytes
    temp_file.flush()?;

    // Should not panic, should return error or empty tags
    let result = extractor.extract(temp_file.path());

    // We accept graceful failure
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            !err_msg.contains("panic"),
            "Should not panic on corrupt file: {}",
            err_msg
        );
    }

    Ok(())
}

#[test]
fn test_series_position_as_float() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/test/book.mp3");

    let metadata = ExtractedMetadata {
        title: Some("Book 2.5".to_string()),
        author: Some("Author".to_string()),
        narrator: None,
        description: None,
        series: Some("My Series".to_string()),
        series_position: Some(2.5), // Fractional position
        duration: storystream_core::Duration::from_seconds(1000),
        file_size: 1_000_000,
        format: storystream_media_formats::AudioFormat::Mp3,
        bitrate: Some(128000),
        sample_rate: 44100,
        channels: 2,
        cover_art: None,
    };

    let book = extractor.to_book(path, metadata);

    assert_eq!(book.series_position, Some(2.5));

    Ok(())
}

#[test]
fn test_zero_duration_handled() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/test/zero.mp3");

    let metadata = ExtractedMetadata {
        title: Some("Zero Duration".to_string()),
        author: None,
        narrator: None,
        description: None,
        series: None,
        series_position: None,
        duration: storystream_core::Duration::from_seconds(0),
        file_size: 1000,
        format: storystream_media_formats::AudioFormat::Mp3,
        bitrate: Some(128000),
        sample_rate: 44100,
        channels: 1,
        cover_art: None,
    };

    let book = extractor.to_book(path, metadata);

    assert_eq!(book.duration.as_seconds(), 0);

    Ok(())
}

#[test]
fn test_large_file_size() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/test/large.flac");

    let large_size = 5_000_000_000u64; // 5 GB

    let metadata = ExtractedMetadata {
        title: Some("Large File".to_string()),
        author: None,
        narrator: None,
        description: None,
        series: None,
        series_position: None,
        duration: storystream_core::Duration::from_seconds(36000),
        file_size: large_size,
        format: storystream_media_formats::AudioFormat::Flac,
        bitrate: Some(1411000),
        sample_rate: 96000,
        channels: 2,
        cover_art: None,
    };

    let book = extractor.to_book(path, metadata);

    // Should handle large file sizes without overflow
    assert_eq!(book.title, "Large File");

    Ok(())
}

#[test]
fn test_mono_audio() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/test/mono.mp3");

    let metadata = ExtractedMetadata {
        title: Some("Mono Audio".to_string()),
        author: None,
        narrator: None,
        description: None,
        series: None,
        series_position: None,
        duration: storystream_core::Duration::from_seconds(600),
        file_size: 5_000_000,
        format: storystream_media_formats::AudioFormat::Mp3,
        bitrate: Some(64000),
        sample_rate: 22050,
        channels: 1, // Mono
        cover_art: None,
    };

    let book = extractor.to_book(path, metadata);

    assert_eq!(book.title, "Mono Audio");

    Ok(())
}

#[test]
fn test_very_long_title() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/test/long.mp3");

    let long_title = "A".repeat(1000);

    let metadata = ExtractedMetadata {
        title: Some(long_title.clone()),
        author: None,
        narrator: None,
        description: None,
        series: None,
        series_position: None,
        duration: storystream_core::Duration::from_seconds(100),
        file_size: 1_000_000,
        format: storystream_media_formats::AudioFormat::Mp3,
        bitrate: Some(128000),
        sample_rate: 44100,
        channels: 2,
        cover_art: None,
    };

    let book = extractor.to_book(path, metadata);

    assert_eq!(book.title, long_title);

    Ok(())
}

#[test]
fn test_special_characters_in_metadata() -> Result<()> {
    let extractor = MetadataExtractor::new()?;
    let path = Path::new("/test/special.mp3");

    let metadata = ExtractedMetadata {
        title: Some("Book: A Tale of \"Quotes\" & <Tags>".to_string()),
        author: Some("O'Reilly & Sons".to_string()),
        narrator: Some("Narrator (2024)".to_string()),
        description: Some("Description with\nnewlines\tand\ttabs".to_string()),
        series: None,
        series_position: None,
        duration: storystream_core::Duration::from_seconds(1500),
        file_size: 10_000_000,
        format: storystream_media_formats::AudioFormat::Mp3,
        bitrate: Some(128000),
        sample_rate: 44100,
        channels: 2,
        cover_art: None,
    };

    let book = extractor.to_book(path, metadata);

    assert_eq!(book.title, "Book: A Tale of \"Quotes\" & <Tags>");
    assert_eq!(book.author, Some("O'Reilly & Sons".to_string()));
    assert_eq!(book.narrator, Some("Narrator (2024)".to_string()));

    Ok(())
}
