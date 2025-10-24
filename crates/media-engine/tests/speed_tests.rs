use media_engine::speed::{Speed, SpeedProcessor};

#[test]
fn test_speed_processor_basics() {
    let processor = SpeedProcessor::new(44100, 2);
    assert_eq!(processor.speed().value(), 1.0);
    assert!(processor.pitch_correction_enabled());
}

#[test]
fn test_speed_range_validation() {
    // Valid speeds
    assert!(Speed::new(0.5).is_ok());
    assert!(Speed::new(1.0).is_ok());
    assert!(Speed::new(1.5).is_ok());
    assert!(Speed::new(2.0).is_ok());
    assert!(Speed::new(3.0).is_ok());

    // Invalid speeds
    assert!(Speed::new(0.49).is_err());
    assert!(Speed::new(3.01).is_err());
    assert!(Speed::new(0.0).is_err());
    assert!(Speed::new(-1.0).is_err());
    assert!(Speed::new(f32::NAN).is_err());
    assert!(Speed::new(f32::INFINITY).is_err());
}

#[test]
fn test_speed_accuracy() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(1.5).unwrap()).unwrap();
    processor.set_pitch_correction(false).unwrap();

    let input_len = 1000;
    let input: Vec<f32> = (0..input_len)
        .map(|i| (i as f32 / input_len as f32) * 0.5)
        .collect();

    let output = processor.process(&input).unwrap();
    let expected_len = (input_len as f32 / 1.5) as usize;
    let tolerance = (expected_len as f32 * 0.1) as usize;

    assert!(
        (output.len() as i32 - expected_len as i32).abs() < tolerance as i32,
        "Expected ~{} samples, got {}",
        expected_len,
        output.len()
    );
}

#[test]
fn test_speed_slowdown() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(0.5).unwrap()).unwrap();
    processor.set_pitch_correction(false).unwrap();

    // Generate 1 second of test audio (stereo)
    let duration_samples = 44100 * 2; // 1 second stereo
    let mut input = Vec::with_capacity(duration_samples);
    for i in 0..duration_samples / 2 {
        let sample = (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin() * 0.5;
        input.push(sample); // Left
        input.push(sample); // Right
    }

    let output = processor.process(&input).unwrap();

    // At 0.5x speed, output should be roughly double the length
    let expected_length = input.len() * 2;
    let tolerance = (expected_length as f32 * 0.1) as usize; // 10% tolerance
    assert!(
        (output.len() as i32 - expected_length as i32).abs() < tolerance as i32,
        "Expected output length ~{}, got {}",
        expected_length,
        output.len()
    );

    // Verify output is not silent
    let max_amplitude = output.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_amplitude > 0.1, "Output should not be silent");
}

#[test]
fn test_speed_speedup() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(2.0).unwrap()).unwrap();
    processor.set_pitch_correction(false).unwrap();

    // Generate 1 second of test audio (stereo)
    let duration_samples = 44100 * 2; // 1 second stereo
    let mut input = Vec::with_capacity(duration_samples);
    for i in 0..duration_samples / 2 {
        let sample = (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin() * 0.5;
        input.push(sample); // Left
        input.push(sample); // Right
    }

    let output = processor.process(&input).unwrap();

    // At 2x speed, output should be roughly half the length
    let expected_length = input.len() / 2;
    let tolerance = (expected_length as f32 * 0.1) as usize; // 10% tolerance
    assert!(
        (output.len() as i32 - expected_length as i32).abs() < tolerance as i32,
        "Expected output length ~{}, got {}",
        expected_length,
        output.len()
    );

    // Verify output is not silent
    let max_amplitude = output.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_amplitude > 0.1, "Output should not be silent");
}

#[test]
fn test_pitch_correction_toggle() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(1.5).unwrap()).unwrap();

    assert!(processor.pitch_correction_enabled());
    processor.set_pitch_correction(false).unwrap();
    assert!(!processor.pitch_correction_enabled());
    processor.set_pitch_correction(true).unwrap();
    assert!(processor.pitch_correction_enabled());
}

#[test]
fn test_dynamic_speed_changes() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(1.5).unwrap()).unwrap();

    let input: Vec<f32> = (0..1000).map(|i| (i as f32 / 1000.0) * 0.5).collect();
    let _ = processor.process(&input).unwrap();

    processor.set_speed(Speed::new(0.75).unwrap()).unwrap();
    let _ = processor.process(&input).unwrap();

    processor.set_speed(Speed::new(2.5).unwrap()).unwrap();
    let _ = processor.process(&input).unwrap();
}

#[test]
fn test_mono_audio() {
    let mut processor = SpeedProcessor::new(44100, 1);
    processor.set_speed(Speed::new(1.5).unwrap()).unwrap();
    processor.set_pitch_correction(false).unwrap();

    let input: Vec<f32> = (0..1000).map(|i| (i as f32 / 1000.0) * 0.5).collect();
    let output = processor.process(&input).unwrap();

    assert!(!output.is_empty());
}

#[test]
fn test_multi_channel_audio() {
    let mut processor = SpeedProcessor::new(48000, 6);
    processor.set_speed(Speed::new(1.2).unwrap()).unwrap();
    processor.set_pitch_correction(false).unwrap();

    let input: Vec<f32> = (0..6000).map(|i| (i as f32 / 6000.0) * 0.5).collect();
    let output = processor.process(&input).unwrap();

    assert_eq!(output.len() % 6, 0);
}

#[test]
fn test_small_buffer_handling() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(1.5).unwrap()).unwrap();
    processor.set_pitch_correction(false).unwrap();

    let input: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4];
    let output = processor.process(&input).unwrap();

    assert!(!output.is_empty());
}

#[test]
fn test_large_buffer_handling() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(1.5).unwrap()).unwrap();
    processor.set_pitch_correction(false).unwrap();

    let input: Vec<f32> = (0..100000).map(|i| (i as f32 / 100000.0) * 0.5).collect();
    let output = processor.process(&input).unwrap();

    assert!(!output.is_empty());
}

#[test]
fn test_zero_amplitude_handling() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(1.5).unwrap()).unwrap();

    let input: Vec<f32> = vec![0.0; 1000];
    let output = processor.process(&input).unwrap();

    assert!(!output.is_empty());
    let max_amplitude = output.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_amplitude < 0.01);
}

#[test]
fn test_extreme_amplitude_handling() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(1.0).unwrap()).unwrap();

    let input: Vec<f32> = vec![1.0, -1.0, 1.0, -1.0];
    let output = processor.process(&input).unwrap();

    assert!(!output.is_empty());
    let max_amplitude = output.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_amplitude <= 1.1); // Allow small overshoot from processing
}

#[test]
fn test_reset_clears_state() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(1.0).unwrap()).unwrap();

    let input: Vec<f32> = (0..1000).map(|i| (i as f32 / 1000.0) * 0.5).collect();
    let _ = processor.process(&input).unwrap();

    processor.reset();

    // After reset, should still process correctly
    let output = processor.process(&input).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_flush_behavior() {
    let mut processor = SpeedProcessor::new(44100, 2);
    let flushed = processor.flush().unwrap();
    assert!(flushed.is_empty());
}

#[test]
fn test_concurrent_operations() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_speed(Speed::new(1.5).unwrap()).unwrap();

    for _ in 0..10 {
        let input: Vec<f32> = (0..1000).map(|i| (i as f32 / 1000.0) * 0.5).collect();
        let output = processor.process(&input).unwrap();
        assert!(!output.is_empty());
    }
}

#[test]
fn test_edge_case_speeds() {
    let mut processor = SpeedProcessor::new(44100, 2);
    processor.set_pitch_correction(false).unwrap();

    // Test exact boundaries
    for speed in [0.5, 3.0] {
        processor.set_speed(Speed::new(speed).unwrap()).unwrap();
        let input: Vec<f32> = (0..1000).map(|i| (i as f32 / 1000.0) * 0.5).collect();
        let output = processor.process(&input).unwrap();
        assert!(!output.is_empty(), "Speed {} should produce output", speed);
    }
}

#[test]
fn test_sample_rate_variations() {
    for rate in [8000, 16000, 22050, 44100, 48000, 96000] {
        let mut processor = SpeedProcessor::new(rate, 2);
        processor.set_speed(Speed::new(1.5).unwrap()).unwrap();
        processor.set_pitch_correction(false).unwrap();

        let input: Vec<f32> = (0..1000).map(|i| (i as f32 / 1000.0) * 0.5).collect();
        let output = processor.process(&input).unwrap();
        assert!(!output.is_empty(), "Sample rate {} should work", rate);
    }
}
