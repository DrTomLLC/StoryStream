// FILE: crates/media-formats/tests/integration_tests.rs

use std::path::PathBuf;
use storystream_media_formats::prelude::*;

#[test]
fn test_format_detection_from_extension() {
    let test_cases = vec![
        ("test.mp3", Some(AudioFormat::Mp3)),
        ("test.m4b", Some(AudioFormat::M4b)),
        ("test.flac", Some(AudioFormat::Flac)),
        ("test.opus", Some(AudioFormat::Opus)),
        ("test.wav", Some(AudioFormat::Wav)),
        ("test.unknown", None),
    ];

    for (filename, expected) in test_cases {
        let path = PathBuf::from(filename);
        let result = AudioFormat::from_path(&path);
        assert_eq!(result, expected, "Failed for {}", filename);
    }
}

#[test]
fn test_audio_quality_scoring() {
    // Lossy standard quality (no bitrate info defaults to Standard)
    let q1 = AudioQuality::new(44_100, 16, false, false, None);
    assert!(q1.score() >= 45 && q1.score() < 60); // Standard tier gets ~50 score

    // CD quality lossless (properly marked as lossless)
    let q2 = AudioQuality::new(44_100, 16, true, false, None);
    assert!(q2.score() >= 80); // CD quality lossless gets 80+

    // Hi-Res 96kHz
    let q3 = AudioQuality::new(96_000, 24, true, false, None);
    assert!(q3.score() >= 90); // Hi-Res gets 92+

    // Hi-res should score higher than CD
    assert!(q3.score() > q2.score());

    // CD should score higher than standard lossy
    assert!(q2.score() > q1.score());
}

#[test]
fn test_audio_analyzer_creation() {
    let result = AudioAnalyzer::new();
    assert!(
        result.is_ok(),
        "Failed to create analyzer: {:?}",
        result.err()
    );
}

#[test]
fn test_format_detection_nonexistent_file() {
    let result = AudioAnalyzer::new();
    assert!(result.is_ok(), "Failed to create analyzer");

    let analyzer = match result {
        Ok(a) => a,
        Err(e) => {
            panic!("Failed to create analyzer: {:?}", e);
        }
    };

    let path = PathBuf::from("/nonexistent/file.mp3");
    let result = analyzer.analyze(&path);
    assert!(result.is_err());

    match result {
        Err(FormatError::FileNotFound { .. }) => {}
        Err(e) => panic!("Expected FileNotFound error, got: {:?}", e),
        Ok(_) => panic!("Expected error for nonexistent file"),
    }
}

#[test]
fn test_unsupported_extension() {
    let path = PathBuf::from("test.xyz");
    let result = AudioFormat::from_path(&path);
    assert!(result.is_none());
}

#[test]
fn test_error_recoverability() {
    let path = PathBuf::from("/test/file.mp3");

    let err1 = FormatError::file_not_found(path.clone());
    assert!(err1.is_recoverable());

    let err2 = FormatError::corrupted(path.clone(), "Bad data");
    assert!(!err2.is_recoverable());
    assert!(err2.is_corruption());

    let err3 = FormatError::UnsupportedFormat("WMA".to_string());
    assert!(err3.is_recoverable());
}

#[test]
fn test_format_capabilities() {
    let caps = FormatCapabilities::for_format(AudioFormat::Flac);
    assert!(caps.cover_art);
    assert!(caps.chapters);
    assert!(caps.seekable);

    let caps = FormatCapabilities::for_format(AudioFormat::Mp3);
    assert!(caps.cover_art);
    assert!(caps.chapters);

    let caps = FormatCapabilities::for_format(AudioFormat::Wav);
    assert!(!caps.cover_art);
    assert!(!caps.chapters);
}

#[test]
fn test_quality_report_generation() {
    let quality = AudioQuality::new(96_000, 24, true, false, None).with_dynamic_range(14.5);

    let report = quality.report();

    // Verify report contains key information
    assert!(report.contains("Hi-Res") || report.contains("96000"));
    assert!(report.contains("24"));
    assert!(report.contains("Lossless"));
    assert!(report.contains("Quality Score"));
}

#[test]
fn test_all_formats_have_extensions() {
    for format in AudioFormat::all() {
        let ext = format.extension();
        assert!(!ext.is_empty());
        assert!(!ext.starts_with('.'));
    }
}

#[test]
fn test_all_formats_have_names() {
    for format in AudioFormat::all() {
        let name = format.name();
        assert!(!name.is_empty());
    }
}

#[test]
fn test_all_formats_have_mime_types() {
    for format in AudioFormat::all() {
        let mime = format.mime_type();
        assert!(!mime.is_empty());
        assert!(mime.starts_with("audio/"));
    }
}

#[test]
fn test_quality_tier_classification() {
    // Low quality
    let q1 = AudioQuality::new(22_050, 8, false, false, Some(64_000));
    assert_eq!(q1.tier, QualityTier::Low); // Fixed: tier is a field, not a method

    // Standard quality
    let q2 = AudioQuality::new(44_100, 16, false, false, Some(128_000));
    assert_eq!(q2.tier, QualityTier::Standard); // Fixed: tier is a field

    // High quality
    let q3 = AudioQuality::new(44_100, 16, false, false, Some(320_000));
    assert_eq!(q3.tier, QualityTier::High); // Fixed: tier is a field

    // CD quality (lossless)
    let q4 = AudioQuality::new(44_100, 16, true, false, None);
    assert_eq!(q4.tier, QualityTier::CD); // Fixed: tier is a field, variant is CD

    // Hi-Res
    let q5 = AudioQuality::new(96_000, 24, true, false, None);
    assert_eq!(q5.tier, QualityTier::HiRes96); // Fixed: variant is HiRes96
}

#[test]
fn test_format_all_returns_all_variants() {
    let formats = AudioFormat::all();
    assert!(formats.len() >= 8); // At least the main formats
    assert!(formats.contains(&AudioFormat::Mp3));
    assert!(formats.contains(&AudioFormat::Flac));
    assert!(formats.contains(&AudioFormat::M4b));
}
