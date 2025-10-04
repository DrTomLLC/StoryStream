// tests/audio_quality_tests.rs

#[test]
fn test_lossless_formats() {
    // Verify all lossless formats decode correctly
    for format in [FLAC, ALAC, WAV, AIFF] {
        let decoder = AudioDecoder::open(format)?;
        assert!(decoder.is_lossless());
    }
}

#[test]
fn test_no_clipping() {
    // Ensure audio never clips during processing
    let samples = process_audio(test_file)?;
    for sample in samples {
        assert!(sample >= -1.0 && sample <= 1.0);
    }
}

#[test]
fn test_gapless_playback() {
    // Verify no gaps between chapters
    let engine = AudioEngine::new();
    engine.load_with_chapters(file)?;

    // Check continuity at chapter boundaries
    for i in 0..engine.chapter_count() - 1 {
        engine.seek_to_chapter(i)?;
        let end_samples = engine.read_to_chapter_end()?;
        let start_samples = engine.read_from_chapter_start(i + 1)?;

        // Should be continuous
        assert_no_gap(end_samples, start_samples);
    }
}

#[test]
fn test_speed_quality() {
    // Ensure speed changes maintain quality
    for speed in [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0] {
        let engine = AudioEngine::new();
        engine.set_speed(speed)?;

        let output = engine.process(test_samples)?;

        // Check for artifacts
        assert_no_aliasing(output);
        assert_frequency_response_preserved(output);
    }
}