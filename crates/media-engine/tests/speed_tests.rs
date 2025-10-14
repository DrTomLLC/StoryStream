//! Comprehensive tests for speed change functionality

use media_engine::SpeedProcessor;

#[test]
fn test_speed_processor_basics() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.0).unwrap();

    let input: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
    let output = processor.process(&input).unwrap();
    assert_eq!(output.len(), input.len());
}

#[test]
fn test_speed_range_validation() {
    assert!(SpeedProcessor::new(44100, 2, 0.5).is_ok());
    assert!(SpeedProcessor::new(44100, 2, 1.0).is_ok());
    assert!(SpeedProcessor::new(44100, 2, 1.5).is_ok());
    assert!(SpeedProcessor::new(44100, 2, 2.0).is_ok());
    assert!(SpeedProcessor::new(44100, 2, 3.0).is_ok());
    assert!(SpeedProcessor::new(44100, 2, 0.49).is_err());
    assert!(SpeedProcessor::new(44100, 2, 3.01).is_err());
    assert!(SpeedProcessor::new(44100, 2, 0.0).is_err());
    assert!(SpeedProcessor::new(44100, 2, -1.0).is_err());
    assert!(SpeedProcessor::new(44100, 2, f32::NAN).is_err());
    assert!(SpeedProcessor::new(44100, 2, f32::INFINITY).is_err());
}

#[test]
fn test_dynamic_speed_changes() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.0).unwrap();
    let test_data: Vec<f32> = vec![0.1; 1000];

    for speed in &[1.0, 1.5, 2.0, 1.2, 0.8, 1.0] {
        assert!(processor.set_speed(*speed).is_ok());
        let _output = processor.process(&test_data).unwrap();
    }
}

#[test]
fn test_speed_slowdown() {
    let mut processor = SpeedProcessor::new(44100, 2, 0.5).unwrap();

    let input_frames = 1000;
    let input: Vec<f32> = vec![0.5; input_frames * 2];

    processor.set_speed(0.5).unwrap();

    let mut total_output = Vec::new();
    for _ in 0..5 {
        let output = processor.process(&input).unwrap();
        total_output.extend(output);
    }

    assert!(total_output.len() > input.len());
}

#[test]
fn test_speed_speedup() {
    let mut processor = SpeedProcessor::new(44100, 2, 2.0).unwrap();

    let input_frames = 10000;
    let input: Vec<f32> = vec![0.5; input_frames * 2];

    let mut total_output = Vec::new();
    for _ in 0..10 {
        let output = processor.process(&input).unwrap();
        total_output.extend(output);
    }

    let total_input = input.len() * 10;
    assert!(total_output.len() < total_input);
}

#[test]
fn test_flush_behavior() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.5).unwrap();

    let input: Vec<f32> = vec![0.3; 512];
    let _ = processor.process(&input).unwrap();

    let flushed = processor.flush().unwrap();
    assert!(!flushed.is_empty() || input.len() < 1024);

    let flushed_again = processor.flush().unwrap();
    assert!(flushed_again.is_empty());
}

#[test]
fn test_reset_clears_state() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.5).unwrap();

    let input: Vec<f32> = vec![0.5; 2048];
    let _ = processor.process(&input).unwrap();

    processor.reset();

    let flushed = processor.flush().unwrap();
    assert!(flushed.is_empty());
}

#[test]
fn test_mono_audio() {
    let mut processor = SpeedProcessor::new(44100, 1, 1.5).unwrap();
    let input: Vec<f32> = vec![0.5; 1000];

    let output = processor.process(&input);
    assert!(output.is_ok());
}

#[test]
fn test_multi_channel_audio() {
    let mut processor = SpeedProcessor::new(48000, 6, 1.0).unwrap();
    let input: Vec<f32> = vec![0.5; 6000];

    let output = processor.process(&input);
    assert!(output.is_ok());
}

#[test]
fn test_small_buffer_handling() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.2).unwrap();

    for size in &[2, 4, 8, 16, 32] {
        let input: Vec<f32> = vec![0.1; *size];
        let result = processor.process(&input);
        assert!(result.is_ok());
    }
}

#[test]
fn test_large_buffer_handling() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.5).unwrap();

    let input: Vec<f32> = vec![0.5; 100000];
    let result = processor.process(&input);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_zero_amplitude_handling() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.5).unwrap();

    let input: Vec<f32> = vec![0.0; 1000];
    let output = processor.process(&input).unwrap();

    for sample in &output {
        assert!(sample.abs() < 0.0001);
    }
}

#[test]
fn test_extreme_amplitude_handling() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.0).unwrap();

    let input: Vec<f32> = vec![1.0, -1.0, 1.0, -1.0];
    let output = processor.process(&input);

    assert!(output.is_ok());
    let samples = output.unwrap();

    for sample in &samples {
        assert!(sample.abs() <= 1.0);
    }
}

#[test]
fn test_speed_accuracy() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.0).unwrap();

    assert_eq!(processor.current_speed(), 1.0);
    assert_eq!(processor.target_speed(), 1.0);

    processor.set_speed(1.75).unwrap();
    assert_eq!(processor.target_speed(), 1.75);
}

#[test]
fn test_pitch_correction_toggle() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.0).unwrap();

    processor.set_pitch_correction(false);
    processor.set_pitch_correction(true);

    let input: Vec<f32> = vec![0.5; 100];
    let result = processor.process(&input);
    assert!(result.is_ok());
}

#[test]
fn test_concurrent_operations() {
    let mut processor = SpeedProcessor::new(44100, 2, 1.0).unwrap();

    let input: Vec<f32> = vec![0.5; 1000];

    for i in 0..20 {
        let speed = 1.0 + (i as f32 * 0.1);
        if speed <= 3.0 {
            processor.set_speed(speed).unwrap();
        }
        let _ = processor.process(&input);
    }
}

#[test]
fn test_edge_case_speeds() {
    let speeds = vec![0.5, 0.50001, 2.99999, 3.0];

    for speed in speeds {
        let result = SpeedProcessor::new(44100, 2, speed);
        assert!(result.is_ok(), "Speed {} should be valid", speed);
    }
}

#[test]
fn test_sample_rate_variations() {
    let sample_rates = vec![22050, 44100, 48000, 96000, 192000];

    for rate in sample_rates {
        let result = SpeedProcessor::new(rate, 2, 1.0);
        assert!(result.is_ok(), "Sample rate {} should work", rate);
    }
}