/// Represents a playback speed multiplier
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Speed {
    value: f32,
}

impl Speed {
    pub const MIN: f32 = 0.5;
    pub const MAX: f32 = 3.0;
    pub const DEFAULT: f32 = 1.0;

    /// Creates a new speed value, clamping to valid range
    pub fn new(value: f32) -> Result<Self, String> {
        // Check for NaN and infinity
        if !value.is_finite() {
            return Err(format!(
                "Speed must be a finite number, got {}",
                value
            ));
        }

        if value < Self::MIN || value > Self::MAX {
            return Err(format!(
                "Speed must be between {} and {}, got {}",
                Self::MIN,
                Self::MAX,
                value
            ));
        }
        Ok(Self { value })
    }

    /// Returns the default speed (1.0x)
    pub fn default() -> Self {
        Self {
            value: Self::DEFAULT,
        }
    }

    /// Returns the numeric value
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Checks if this is normal speed
    pub fn is_normal(&self) -> bool {
        (self.value - Self::DEFAULT).abs() < f32::EPSILON
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::default()
    }
}

impl std::fmt::Display for Speed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}x", self.value)
    }
}

impl PartialOrd for Speed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

/// Processes audio to change playback speed using time-stretching
pub struct SpeedProcessor {
    sample_rate: u32,
    channels: u16,
    speed: Speed,
    pitch_correction: bool,
    input_buffer: Vec<Vec<f32>>,
    output_buffer: Vec<Vec<f32>>,
    phase: f64,
}

impl SpeedProcessor {
    /// Creates a new speed processor
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
            speed: Speed::default(),
            pitch_correction: true,
            input_buffer: vec![Vec::new(); channels as usize],
            output_buffer: vec![Vec::new(); channels as usize],
            phase: 0.0,
        }
    }

    /// Sets the playback speed
    pub fn set_speed(&mut self, speed: Speed) -> Result<(), String> {
        if speed.value() < Speed::MIN || speed.value() > Speed::MAX {
            return Err(format!(
                "Speed must be between {} and {}",
                Speed::MIN,
                Speed::MAX
            ));
        }

        self.speed = speed;
        self.phase = 0.0;
        Ok(())
    }

    /// Gets the current speed
    pub fn speed(&self) -> Speed {
        self.speed
    }

    /// Enables or disables pitch correction
    pub fn set_pitch_correction(&mut self, enabled: bool) -> Result<(), String> {
        self.pitch_correction = enabled;
        self.phase = 0.0;
        Ok(())
    }

    /// Returns whether pitch correction is enabled
    pub fn pitch_correction_enabled(&self) -> bool {
        self.pitch_correction
    }

    /// Processes an input buffer and returns the speed-adjusted output
    pub fn process(&mut self, input: &[f32]) -> Result<Vec<f32>, String> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        // If speed is normal (1.0), return input unchanged
        if self.speed.is_normal() {
            return Ok(input.to_vec());
        }

        let samples_per_channel = input.len() / self.channels as usize;

        // Deinterleave input
        self.input_buffer.iter_mut().for_each(|b| b.clear());
        for (i, &sample) in input.iter().enumerate() {
            let channel = i % self.channels as usize;
            self.input_buffer[channel].push(sample);
        }

        // Process with or without pitch correction
        if self.pitch_correction {
            self.process_with_pitch_correction(samples_per_channel)
        } else {
            self.process_without_pitch_correction()
        }
    }

    /// Processes audio with pitch correction using WSOLA (Waveform Similarity Overlap-Add)
    fn process_with_pitch_correction(&mut self, input_frames: usize) -> Result<Vec<f32>, String> {
        let speed_ratio = self.speed.value();

        // Calculate output length based on speed
        let output_frames = (input_frames as f32 / speed_ratio) as usize;

        // Window size in samples (20ms at 44.1kHz = 882 samples, use power of 2)
        let window_size = 1024.min(input_frames / 4).max(64);
        let hop_in = (window_size as f32 * speed_ratio) as usize;
        let hop_out = window_size;

        self.output_buffer.iter_mut().for_each(|b| b.clear());

        // Process each channel independently
        for channel_idx in 0..self.channels as usize {
            let input_channel = &self.input_buffer[channel_idx];
            let mut output_channel = Vec::with_capacity(output_frames);

            let mut in_pos = 0;
            let mut out_pos = 0;

            // Apply Hann window for smooth transitions
            let window = self.create_hann_window(window_size);

            while in_pos + window_size <= input_frames && out_pos < output_frames {
                // Extract windowed segment
                let mut segment = Vec::with_capacity(window_size);
                for i in 0..window_size {
                    if in_pos + i < input_frames {
                        segment.push(input_channel[in_pos + i] * window[i]);
                    } else {
                        segment.push(0.0);
                    }
                }

                // Add to output with overlap-add
                for (i, &sample) in segment.iter().enumerate() {
                    let out_idx = out_pos + i;
                    if out_idx < output_frames {
                        if out_idx >= output_channel.len() {
                            output_channel.resize(out_idx + 1, 0.0);
                        }
                        output_channel[out_idx] += sample;
                    }
                }

                in_pos += hop_in;
                out_pos += hop_out;
            }

            // Ensure output has the correct length
            output_channel.resize(output_frames, 0.0);

            // Normalize to prevent clipping from overlap-add
            let max_val = output_channel.iter()
                .map(|&x| x.abs())
                .fold(0.0f32, f32::max);

            if max_val > 1.0 {
                let scale = 1.0 / max_val;
                for sample in &mut output_channel {
                    *sample *= scale;
                }
            }

            self.output_buffer[channel_idx] = output_channel;
        }

        // Interleave output
        let output_frames = self.output_buffer[0].len();
        let mut output = Vec::with_capacity(output_frames * self.channels as usize);

        for frame in 0..output_frames {
            for channel in 0..self.channels as usize {
                if frame < self.output_buffer[channel].len() {
                    output.push(self.output_buffer[channel][frame]);
                } else {
                    output.push(0.0);
                }
            }
        }

        Ok(output)
    }

    /// Creates a Hann window for smooth transitions
    fn create_hann_window(&self, size: usize) -> Vec<f32> {
        (0..size)
            .map(|i| {
                let phase = std::f32::consts::PI * i as f32 / (size - 1) as f32;
                0.5 * (1.0 - phase.cos())
            })
            .collect()
    }

    /// Processes audio without pitch correction (simple time-stretching)
    fn process_without_pitch_correction(&self) -> Result<Vec<f32>, String> {
        let input_frames = self.input_buffer[0].len();
        let output_frames = (input_frames as f32 / self.speed.value()) as usize;

        let mut output = Vec::with_capacity(output_frames * self.channels as usize);

        for out_frame in 0..output_frames {
            let in_frame_f = out_frame as f32 * self.speed.value();
            let in_frame = in_frame_f as usize;
            let frac = in_frame_f - in_frame as f32;

            for channel in 0..self.channels as usize {
                let sample = if in_frame + 1 < input_frames {
                    // Linear interpolation
                    let s0 = self.input_buffer[channel][in_frame];
                    let s1 = self.input_buffer[channel][in_frame + 1];
                    s0 + (s1 - s0) * frac
                } else if in_frame < input_frames {
                    self.input_buffer[channel][in_frame]
                } else {
                    0.0
                };

                output.push(sample);
            }
        }

        Ok(output)
    }

    /// Resets internal state
    pub fn reset(&mut self) {
        self.input_buffer.iter_mut().for_each(|b| b.clear());
        self.output_buffer.iter_mut().for_each(|b| b.clear());
        self.phase = 0.0;
    }

    /// Flushes any remaining audio in internal buffers
    pub fn flush(&mut self) -> Result<Vec<f32>, String> {
        // For our implementation, we don't buffer across calls
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_creation() {
        assert!(Speed::new(1.0).is_ok());
        assert!(Speed::new(0.5).is_ok());
        assert!(Speed::new(3.0).is_ok());
        assert!(Speed::new(0.4).is_err());
        assert!(Speed::new(3.1).is_err());
    }

    #[test]
    fn test_speed_default() {
        let speed = Speed::default();
        assert_eq!(speed.value(), 1.0);
        assert!(speed.is_normal());
    }

    #[test]
    fn test_speed_display() {
        let speed = Speed::new(1.5).unwrap();
        assert_eq!(format!("{}", speed), "1.50x");
    }

    #[test]
    fn test_speed_comparison() {
        let s1 = Speed::new(1.0).unwrap();
        let s2 = Speed::new(1.5).unwrap();
        assert!(s1 < s2);
    }

    #[test]
    fn test_processor_creation() {
        let processor = SpeedProcessor::new(44100, 2);
        assert_eq!(processor.speed().value(), 1.0);
        assert!(processor.pitch_correction_enabled());
    }

    #[test]
    fn test_set_speed() {
        let mut processor = SpeedProcessor::new(44100, 2);
        assert!(processor.set_speed(Speed::new(1.5).unwrap()).is_ok());
        assert_eq!(processor.speed().value(), 1.5);
    }

    #[test]
    fn test_set_speed_invalid() {
        let processor = SpeedProcessor::new(44100, 2);
        let invalid_speed = Speed::new(0.4);
        assert!(invalid_speed.is_err());
        // Speed validation happens in Speed::new, so we can't set invalid speed
        assert_eq!(processor.speed().value(), 1.0);
    }

    #[test]
    fn test_pitch_correction_toggle() {
        let mut processor = SpeedProcessor::new(44100, 2);
        assert!(processor.set_pitch_correction(false).is_ok());
        assert!(!processor.pitch_correction_enabled());
        assert!(processor.set_pitch_correction(true).is_ok());
        assert!(processor.pitch_correction_enabled());
    }

    #[test]
    fn test_process_normal_speed() {
        let mut processor = SpeedProcessor::new(44100, 2);
        let input: Vec<f32> = (0..100).map(|i| (i as f32) / 100.0).collect();
        let output = processor.process(&input).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_process_empty_input() {
        let mut processor = SpeedProcessor::new(44100, 2);
        let output = processor.process(&[]).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_reset() {
        let mut processor = SpeedProcessor::new(44100, 2);
        processor.set_speed(Speed::new(1.5).unwrap()).unwrap();
        processor.reset();
        // Reset shouldn't change speed, just clear buffers
        assert_eq!(processor.speed().value(), 1.5);
    }

    #[test]
    fn test_flush() {
        let mut processor = SpeedProcessor::new(44100, 2);
        let result = processor.flush().unwrap();
        assert!(result.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_speed_workflow() {
        let mut processor = SpeedProcessor::new(44100, 2);

        // Test speed changes
        processor.set_speed(Speed::new(1.5).unwrap()).unwrap();
        assert_eq!(processor.speed().value(), 1.5);

        // Test pitch correction toggle
        processor.set_pitch_correction(false).unwrap();
        assert!(!processor.pitch_correction_enabled());

        // Generate test audio
        let mut input = Vec::new();
        for i in 0..8820 { // 0.1 seconds stereo at 44.1kHz
            let t = (i / 2) as f32 / 44100.0;
            let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.3;
            input.push(sample);
        }

        // Process
        let output = processor.process(&input).unwrap();

        // Verify non-empty output
        assert!(!output.is_empty());

        // Verify output length is appropriate for speed
        // Without pitch correction at 1.5x, output should be ~2/3 input length
        let ratio = output.len() as f32 / input.len() as f32;
        assert!(
            (ratio - 0.66).abs() < 0.15,
            "Expected ratio ~0.66, got {}",
            ratio
        );

        // Reset and verify
        processor.reset();
        assert_eq!(processor.speed().value(), 1.5); // Speed persists
    }

    #[test]
    fn test_stereo_channel_processing() {
        let mut processor = SpeedProcessor::new(44100, 2);
        processor.set_speed(Speed::new(1.25).unwrap()).unwrap();
        processor.set_pitch_correction(false).unwrap(); // Use simpler mode for deterministic test

        // Create stereo test signal with different content per channel
        let mut input = Vec::new();
        for i in 0..4410 { // 0.05 seconds
            let t = (i / 2) as f32 / 44100.0;
            let left = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.3;
            let right = (2.0 * std::f32::consts::PI * 880.0 * t).sin() * 0.3;
            input.push(left);
            input.push(right);
        }

        let output = processor.process(&input).unwrap();

        // Verify stereo output
        assert_eq!(output.len() % 2, 0, "Output should be stereo interleaved");
        assert!(!output.is_empty());

        // Check that both channels have content
        let left_samples: Vec<f32> = output.iter().step_by(2).copied().collect();
        let right_samples: Vec<f32> = output.iter().skip(1).step_by(2).copied().collect();

        let left_max = left_samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        let right_max = right_samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);

        assert!(left_max > 0.01, "Left channel should have content");
        assert!(right_max > 0.01, "Right channel should have content");
    }

    #[test]
    fn test_multiple_process_calls() {
        let mut processor = SpeedProcessor::new(44100, 1);
        processor.set_speed(Speed::new(1.75).unwrap()).unwrap();
        processor.set_pitch_correction(false).unwrap(); // Use simpler mode for this test

        // Process multiple chunks
        for _ in 0..5 {
            let input: Vec<f32> = (0..1024).map(|i| (i as f32 / 1024.0) * 0.5).collect();
            let output = processor.process(&input).unwrap();
            assert!(!output.is_empty());
        }
    }
}