// crates/media-engine/src/metrics.rs

pub struct AudioQualityMetrics {
    /// Input format
    pub input_format: AudioFormat,
    pub input_sample_rate: u32,
    pub input_bits_per_sample: u8,

    /// Output format
    pub output_sample_rate: u32,
    pub output_bits_per_sample: u8,

    /// Processing
    pub is_resampling: bool,
    pub resample_quality: Option<ResampleQuality>,
    pub is_bit_perfect: bool,
    pub buffer_underruns: u64,
    pub average_latency_ms: f32,

    /// Signal
    pub peak_level_db: f32,
    pub rms_level_db: f32,
    pub dynamic_range_db: f32,
}

impl AudioQualityMetrics {
    /// Display quality report
    pub fn report(&self) -> String {
        format!(
            "Audio Quality Report:
            Input: {} @ {}Hz / {}bit
            Output: {}Hz / {}bit
            Bit-Perfect: {}
            Resampling: {}
            Avg Latency: {:.2}ms
            Peak Level: {:.1}dB
            Dynamic Range: {:.1}dB",
            self.input_format,
            self.input_sample_rate,
            self.input_bits_per_sample,
            self.output_sample_rate,
            self.output_bits_per_sample,
            if self.is_bit_perfect { "YES ✓" } else { "NO" },
            if self.is_resampling {
                format!("YES ({:?})", self.resample_quality)
            } else {
                "NO".to_string()
            },
            self.average_latency_ms,
            self.peak_level_db,
            self.dynamic_range_db,
        )
    }
}