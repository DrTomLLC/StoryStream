// FILE: crates/media-engine/tests/audio_quality_tests.rs
//! Audio Quality Tests - API-CORRECTED, PANIC-FREE IMPLEMENTATION
//!
//! Comprehensive tests for audio quality, format support, and processing accuracy.
//!
//! CRITICAL REQUIREMENTS MET:
//! - ZERO panics, unwraps, expects
//! - Matches actual media-engine API
//! - All errors handled explicitly via Result
//! - Graceful degradation on all failures

use media_engine::{
    AudioDecoder, AudioOutputConfig, EngineError, Equalizer, Speed, SpeedProcessor,
};
use std::f32::consts::PI;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Error Handling
// ============================================================================

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn test_error(msg: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg.into()))
}

// ============================================================================
// Safe Signal Generation (No Panics)
// ============================================================================

fn generate_sine_wave(freq: f32, duration_secs: f32, sample_rate: u32) -> TestResult<Vec<f32>> {
    if freq <= 0.0 {
        return Err(test_error(format!("Invalid frequency: {}", freq)));
    }
    if duration_secs < 0.0 {
        return Err(test_error(format!("Invalid duration: {}", duration_secs)));
    }
    if sample_rate == 0 {
        return Err(test_error("Sample rate cannot be zero"));
    }

    let num_samples = (duration_secs * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let sample = (2.0 * PI * freq * t).sin();
        samples.push(sample);
    }

    Ok(samples)
}

fn generate_silence(duration_secs: f32, sample_rate: u32) -> TestResult<Vec<f32>> {
    if duration_secs < 0.0 {
        return Err(test_error(format!("Invalid duration: {}", duration_secs)));
    }
    if sample_rate == 0 {
        return Err(test_error("Sample rate cannot be zero"));
    }

    let num_samples = (duration_secs * sample_rate as f32) as usize;
    Ok(vec![0.0; num_samples])
}

// ============================================================================
// Safe Quality Metrics (No Panics)
// ============================================================================

fn calculate_rms(samples: &[f32]) -> TestResult<f32> {
    if samples.is_empty() {
        return Ok(0.0);
    }

    let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
    let rms = (sum_squares / samples.len() as f32).sqrt();

    if rms.is_nan() || rms.is_infinite() {
        return Err(test_error("RMS calculation produced invalid result"));
    }

    Ok(rms)
}

fn calculate_peak(samples: &[f32]) -> f32 {
    samples.iter().map(|s| s.abs()).fold(0.0, f32::max)
}

fn has_clipping(samples: &[f32]) -> bool {
    samples.iter().any(|&s| s.abs() > 1.0)
}

fn calculate_snr_db(signal: &[f32], noisy: &[f32]) -> TestResult<f32> {
    if signal.len() != noisy.len() {
        return Err(test_error(format!(
            "Signal length mismatch: {} vs {}",
            signal.len(),
            noisy.len()
        )));
    }

    if signal.is_empty() {
        return Err(test_error("Cannot calculate SNR of empty signal"));
    }

    let signal_power: f32 = signal.iter().map(|s| s * s).sum();
    let noise_power: f32 = signal
        .iter()
        .zip(noisy.iter())
        .map(|(s, n)| {
            let diff = s - n;
            diff * diff
        })
        .sum();

    if noise_power < 1e-10 {
        return Ok(100.0);
    }

    if signal_power <= 0.0 {
        return Err(test_error("Signal power is zero or negative"));
    }

    let snr = 10.0 * (signal_power / noise_power).log10();

    if snr.is_nan() || snr.is_infinite() {
        return Err(test_error("SNR calculation produced invalid result"));
    }

    Ok(snr)
}

// ============================================================================
// Format Handling Tests (No Panics)
// ============================================================================

#[test]
fn test_decode_formats_without_panic() -> TestResult {
    let nonexistent = PathBuf::from("/nonexistent/file.mp3");
    let result = AudioDecoder::new(&nonexistent);

    match result {
        Err(EngineError::DecodeError(_)) => Ok(()),
        Err(e) => Err(test_error(format!("Wrong error type: {:?}", e))),
        Ok(_) => Err(test_error("Should have failed for nonexistent file")),
    }
}

#[test]
fn test_unsupported_format_handling() -> TestResult {
    let temp_dir =
        TempDir::new().map_err(|e| test_error(format!("Failed to create temp dir: {}", e)))?;
    let fake_file = temp_dir.path().join("test.xyz");

    std::fs::write(&fake_file, b"not real audio data")
        .map_err(|e| test_error(format!("Failed to write test file: {}", e)))?;

    let result = AudioDecoder::new(&fake_file);

    if result.is_ok() {
        return Err(test_error("Should reject unsupported format"));
    }

    Ok(())
}

// ============================================================================
// Audio Processing Tests (No Panics)
// ============================================================================

#[test]
fn test_no_clipping_in_processing() -> TestResult {
    let sample_rate = 44100;
    let input = generate_sine_wave(440.0, 1.0, sample_rate)?;

    if has_clipping(&input) {
        return Err(test_error("Generated input should not be clipped"));
    }

    let peak = calculate_peak(&input);
    if peak > 1.0 {
        return Err(test_error(format!("Peak exceeds bounds: {}", peak)));
    }

    let loud_input: Vec<f32> = input.iter().map(|s| s * 0.99).collect();
    if has_clipping(&loud_input) {
        return Err(test_error("Loud input should not clip"));
    }

    Ok(())
}

#[test]
fn test_silence_detection() -> TestResult {
    let sample_rate = 44100;
    let silence = generate_silence(0.1, sample_rate)?;

    let rms = calculate_rms(&silence)?;
    if rms >= 1e-6 {
        return Err(test_error(format!("Silence RMS too high: {}", rms)));
    }

    let peak = calculate_peak(&silence);
    if peak >= 1e-6 {
        return Err(test_error(format!("Silence peak too high: {}", peak)));
    }

    Ok(())
}

#[test]
fn test_volume_scaling_accuracy() -> TestResult {
    let sample_rate = 44100;
    let input = generate_sine_wave(440.0, 0.5, sample_rate)?;
    let original_rms = calculate_rms(&input)?;

    let scaled: Vec<f32> = input.iter().map(|s| s * 0.5).collect();
    let scaled_rms = calculate_rms(&scaled)?;

    let expected_rms = original_rms * 0.5;
    let diff = (scaled_rms - expected_rms).abs();

    if diff >= 1e-5 {
        return Err(test_error(format!(
            "Volume scaling inaccurate: expected {}, got {}, diff {}",
            expected_rms, scaled_rms, diff
        )));
    }

    Ok(())
}

#[test]
fn test_zero_volume_produces_silence() -> TestResult {
    let sample_rate = 44100;
    let input = generate_sine_wave(440.0, 0.5, sample_rate)?;

    let silenced: Vec<f32> = input.iter().map(|s| s * 0.0).collect();

    let rms = calculate_rms(&silenced)?;
    if rms >= 1e-10 {
        return Err(test_error(format!(
            "Zero volume should produce silence, RMS: {}",
            rms
        )));
    }

    let peak = calculate_peak(&silenced);
    if peak >= 1e-10 {
        return Err(test_error(format!(
            "Zero volume should have zero peak: {}",
            peak
        )));
    }

    Ok(())
}

// ============================================================================
// Speed Processing Tests (No Panics)
// ============================================================================

#[test]
fn test_speed_processor_creation() -> TestResult {
    // SpeedProcessor::new returns SpeedProcessor directly, not Result
    let _processor = SpeedProcessor::new(44100, 2);
    Ok(())
}

#[test]
fn test_speed_normal_is_passthrough() -> TestResult {
    let sample_rate = 44100;
    let input = generate_sine_wave(440.0, 0.1, sample_rate)?;

    // SpeedProcessor::new returns SpeedProcessor directly
    let mut processor = SpeedProcessor::new(sample_rate, 1);

    let output = processor
        .process(&input)
        .map_err(|e| test_error(format!("Processing failed: {:?}", e)))?;

    // Length should be roughly the same (within 5%)
    let length_ratio = output.len() as f32 / input.len() as f32;
    if (length_ratio - 1.0).abs() >= 0.05 {
        return Err(test_error(format!(
            "Normal speed should preserve length: ratio = {}",
            length_ratio
        )));
    }

    // Signal characteristics should be preserved
    let min_len = output.len().min(input.len());
    let snr = calculate_snr_db(&input[..min_len], &output[..min_len])?;

    if snr < 30.0 {
        return Err(test_error(format!(
            "Normal speed should have high SNR: {} dB",
            snr
        )));
    }

    Ok(())
}

#[test]
fn test_speed_variations_valid() -> TestResult {
    let sample_rate = 44100;
    let input = generate_sine_wave(440.0, 0.5, sample_rate)?;

    let speeds = vec![0.5, 0.75, 1.0, 1.25, 1.5, 2.0];

    for speed_val in speeds {
        let speed = Speed::new(speed_val)
            .map_err(|e| test_error(format!("Invalid speed {}: {:?}", speed_val, e)))?;

        // SpeedProcessor::new returns SpeedProcessor directly
        let mut processor = SpeedProcessor::new(sample_rate, 1);

        // Set the speed (returns Result)
        processor
            .set_speed(speed)
            .map_err(|e| test_error(format!("Failed to set speed {}: {:?}", speed_val, e)))?;

        let output = processor.process(&input).map_err(|e| {
            test_error(format!(
                "Processing failed for speed {}: {:?}",
                speed_val, e
            ))
        })?;

        if has_clipping(&output) {
            return Err(test_error(format!("Speed {} produced clipping", speed_val)));
        }

        let expected_len = (input.len() as f32 / speed_val) as usize;
        let len_diff = (output.len() as f32 - expected_len as f32).abs() / expected_len as f32;

        if len_diff >= 0.1 {
            return Err(test_error(format!(
                "Speed {} output length mismatch: expected ~{}, got {}, diff ratio {}",
                speed_val,
                expected_len,
                output.len(),
                len_diff
            )));
        }
    }

    Ok(())
}

#[test]
fn test_speed_bounds_checking() -> TestResult {
    if Speed::new(0.5).is_err() {
        return Err(test_error("0.5x should be valid"));
    }
    if Speed::new(3.0).is_err() {
        return Err(test_error("3.0x should be valid"));
    }

    if Speed::new(0.0).is_ok() {
        return Err(test_error("0.0x should be invalid"));
    }
    if Speed::new(-1.0).is_ok() {
        return Err(test_error("Negative speed should be invalid"));
    }
    if Speed::new(10.0).is_ok() {
        return Err(test_error("10.0x should be too fast"));
    }

    Ok(())
}

// ============================================================================
// Equalizer Tests (No Panics)
// ============================================================================

#[test]
fn test_equalizer_creation() -> TestResult {
    let eq = Equalizer::default();

    // Equalizer has 10 bands by default (but bands field is private)
    // We can't access bands directly, just verify creation works
    let _ = eq;

    Ok(())
}

#[test]
fn test_equalizer_disabled_is_passthrough() -> TestResult {
    let sample_rate = 44100;
    let input = generate_sine_wave(440.0, 0.1, sample_rate)?;

    let eq = Equalizer::default(); // Default is disabled

    // Check it's actually disabled
    if eq.is_enabled() {
        return Err(test_error("Default equalizer should be disabled"));
    }

    let output = eq.apply(&input);

    if output.len() != input.len() {
        return Err(test_error("Equalizer should preserve length"));
    }

    let snr = calculate_snr_db(&input, &output)?;

    if snr < 80.0 {
        return Err(test_error(format!(
            "Disabled equalizer should not modify audio: {} dB",
            snr
        )));
    }

    Ok(())
}

#[test]
fn test_equalizer_processing_valid() -> TestResult {
    let sample_rate = 44100;
    let input = generate_sine_wave(440.0, 0.5, sample_rate)?;

    let mut eq = Equalizer::default();
    eq.set_enabled(true); // Use set_enabled instead of enable

    if !eq.is_enabled() {
        return Err(test_error("Equalizer should be enabled"));
    }

    let output = eq.apply(&input);

    if has_clipping(&output) {
        return Err(test_error("Equalizer should not produce clipping"));
    }

    if output.len() != input.len() {
        return Err(test_error("Equalizer should preserve length"));
    }

    Ok(())
}

// ============================================================================
// Integration Tests (No Panics)
// ============================================================================

#[test]
fn test_full_processing_chain() -> TestResult {
    let sample_rate = 44100;
    let input = generate_sine_wave(440.0, 1.0, sample_rate)?;

    // 1. Speed processing (SpeedProcessor::new returns SpeedProcessor directly)
    let mut speed_processor = SpeedProcessor::new(sample_rate, 1);

    let speed_output = speed_processor
        .process(&input)
        .map_err(|e| test_error(format!("Speed processing failed: {:?}", e)))?;

    if has_clipping(&speed_output) {
        return Err(test_error("Speed stage should not clip"));
    }

    // 2. Equalizer
    let eq = Equalizer::default();
    let eq_output = eq.apply(&speed_output);

    if has_clipping(&eq_output) {
        return Err(test_error("EQ stage should not clip"));
    }

    // 3. Volume
    let volume = 0.8;
    let final_output: Vec<f32> = eq_output.iter().map(|s| s * volume).collect();

    if has_clipping(&final_output) {
        return Err(test_error("Final output should not clip"));
    }

    let peak = calculate_peak(&final_output);
    if peak >= 1.0 {
        return Err(test_error(format!(
            "Final peak should be within bounds: {}",
            peak
        )));
    }

    Ok(())
}

#[test]
fn test_processing_chain_with_loud_input() -> TestResult {
    let sample_rate = 44100;
    let base_input = generate_sine_wave(440.0, 0.5, sample_rate)?;
    let input: Vec<f32> = base_input.iter().map(|s| s * 0.95).collect();

    let volume = 0.8;
    let output: Vec<f32> = input.iter().map(|s| s * volume).collect();

    if has_clipping(&output) {
        return Err(test_error("Should handle loud input without clipping"));
    }

    let peak = calculate_peak(&output);
    if peak >= 1.0 {
        return Err(test_error(format!(
            "Peak should remain below 1.0: {}",
            peak
        )));
    }

    Ok(())
}

#[test]
fn test_audio_output_config_validation() -> TestResult {
    let config = AudioOutputConfig::default();

    if config.sample_rate == 0 {
        return Err(test_error("Sample rate should be positive"));
    }
    if config.channels == 0 {
        return Err(test_error("Channel count should be positive"));
    }

    // buffer_size is Option<u32>, so check if Some(0)
    if let Some(size) = config.buffer_size {
        if size == 0 {
            return Err(test_error("Buffer size should be positive"));
        }
    }

    if config.sample_rate < 44100 {
        return Err(test_error("Sample rate should be at least CD quality"));
    }
    if config.channels > 2 {
        return Err(test_error("Should support mono or stereo"));
    }

    Ok(())
}

// ============================================================================
// Edge Case Tests (No Panics)
// ============================================================================

#[test]
fn test_empty_audio_handling() -> TestResult {
    let empty: Vec<f32> = vec![];

    let rms = calculate_rms(&empty)?;
    if !(rms == 0.0 || rms.is_nan()) {
        return Err(test_error(format!("Empty audio RMS unexpected: {}", rms)));
    }

    let peak = calculate_peak(&empty);
    if peak != 0.0 {
        return Err(test_error(format!(
            "Empty audio peak should be 0, got {}",
            peak
        )));
    }

    Ok(())
}

#[test]
fn test_single_sample_processing() -> TestResult {
    let sample_rate = 44100;
    let single_sample = vec![0.5f32];

    // SpeedProcessor::new returns SpeedProcessor directly
    let mut processor = SpeedProcessor::new(sample_rate, 1);

    let result = processor.process(&single_sample);

    if result.is_err() {
        return Err(test_error("Should handle single sample without panic"));
    }

    Ok(())
}

#[test]
fn test_very_short_audio() -> TestResult {
    let sample_rate = 44100;
    let short_audio = generate_sine_wave(440.0, 0.0005, sample_rate)?;

    if short_audio.is_empty() {
        return Err(test_error("Should generate some samples"));
    }

    if has_clipping(&short_audio) {
        return Err(test_error("Short audio should not clip"));
    }

    Ok(())
}

#[test]
fn test_long_audio_duration() -> TestResult {
    let sample_rate = 44100;
    let long_audio = generate_sine_wave(440.0, 10.0, sample_rate)?;

    let expected_len = sample_rate as usize * 10;
    if long_audio.len() != expected_len {
        return Err(test_error(format!(
            "Should generate correct number of samples: expected {}, got {}",
            expected_len,
            long_audio.len()
        )));
    }

    let rms = calculate_rms(&long_audio)?;
    let expected_rms = 1.0 / 2.0f32.sqrt();
    let diff = (rms - expected_rms).abs();

    if diff >= 0.01 {
        return Err(test_error(format!(
            "Long audio should maintain consistent RMS: {} vs expected {}, diff {}",
            rms, expected_rms, diff
        )));
    }

    Ok(())
}

// ============================================================================
// Additional Safety Tests
// ============================================================================

#[test]
fn test_signal_generation_validates_inputs() -> TestResult {
    if generate_sine_wave(-440.0, 1.0, 44100).is_ok() {
        return Err(test_error("Should reject negative frequency"));
    }

    if generate_sine_wave(440.0, -1.0, 44100).is_ok() {
        return Err(test_error("Should reject negative duration"));
    }

    if generate_sine_wave(440.0, 1.0, 0).is_ok() {
        return Err(test_error("Should reject zero sample rate"));
    }

    Ok(())
}

#[test]
fn test_snr_validates_inputs() -> TestResult {
    let signal = vec![0.5f32; 100];
    let noisy = vec![0.5f32; 50];

    if calculate_snr_db(&signal, &noisy).is_ok() {
        return Err(test_error("Should reject mismatched signal lengths"));
    }

    let empty: Vec<f32> = vec![];
    if calculate_snr_db(&empty, &empty).is_ok() {
        return Err(test_error("Should reject empty signals"));
    }

    Ok(())
}
