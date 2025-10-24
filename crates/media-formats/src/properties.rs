//! Audio properties extraction using Symphonia

use crate::{AudioFormat, AudioQuality, FormatError, FormatResult, QualityTier};
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Complete audio file properties
#[derive(Debug, Clone, PartialEq)]
pub struct AudioProperties {
    /// Detected audio format
    pub format: AudioFormat,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1=mono, 2=stereo, etc.)
    pub channels: u8,
    /// Bits per sample
    pub bits_per_sample: u8,
    /// Total duration
    pub duration: Option<Duration>,
    /// Bitrate in bits per second (for lossy formats)
    pub bitrate: Option<u32>,
    /// Is variable bitrate
    pub is_variable_bitrate: bool,
    /// Quality assessment
    pub quality: AudioQuality,
    /// Quality tier
    pub quality_tier: QualityTier,
    /// Codec information
    pub codec: CodecInfo,
    /// Total number of samples
    pub total_samples: Option<u64>,
    /// File size in bytes
    pub file_size: u64,
}

impl AudioProperties {
    /// Returns true if this is high-fidelity audio
    pub fn is_high_fidelity(&self) -> bool {
        self.quality_tier.is_high_fidelity()
    }

    /// Returns true if this is audiophile quality
    pub fn is_audiophile(&self) -> bool {
        self.quality_tier.is_audiophile()
    }

    /// Returns estimated uncompressed size in bytes
    pub fn uncompressed_size(&self) -> Option<u64> {
        self.duration.map(|dur| {
            let samples = dur.as_secs_f64() * self.sample_rate as f64;
            let bytes_per_sample = (self.bits_per_sample / 8) as f64;
            (samples * bytes_per_sample * self.channels as f64) as u64
        })
    }

    /// Returns compression ratio (if compressed)
    pub fn compression_ratio(&self) -> Option<f32> {
        if self.format.is_uncompressed() {
            return Some(1.0);
        }

        self.uncompressed_size()
            .map(|uncompressed| uncompressed as f32 / self.file_size as f32)
    }

    /// Generates a detailed properties report
    pub fn report(&self) -> String {
        let mut report = format!(
            "Audio File Properties\n\
             =====================\n\
             Format: {}\n\
             Codec: {}\n\n\
             {}",
            self.format,
            self.codec.name,
            self.quality.report()
        );

        if let Some(duration) = self.duration {
            report.push_str(&format!("\nDuration: {}", format_duration(duration)));
        }

        report.push_str(&format!(
            "\nChannels: {}\n\
             File Size: {}",
            self.channels,
            format_size(self.file_size)
        ));

        if let Some(ratio) = self.compression_ratio() {
            report.push_str(&format!("\nCompression Ratio: {:.1}:1", ratio));
        }

        report
    }
}

/// Codec information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecInfo {
    /// Codec name
    pub name: String,
    /// Codec type (e.g., "FLAC", "MP3", "AAC")
    pub codec_type: String,
    /// Is lossless
    pub is_lossless: bool,
}

impl CodecInfo {
    fn from_params(params: &CodecParameters, format: AudioFormat) -> Self {
        let codec_type = format!("{:?}", params.codec);
        let name = format.name().to_string();
        let is_lossless = format.is_lossless() || format.is_uncompressed();

        Self {
            name,
            codec_type,
            is_lossless,
        }
    }
}

/// Audio analyzer using Symphonia
pub struct AudioAnalyzer {
    format_opts: FormatOptions,
    metadata_opts: MetadataOptions,
}

impl AudioAnalyzer {
    /// Creates a new audio analyzer
    pub fn new() -> FormatResult<Self> {
        Ok(Self {
            format_opts: FormatOptions::default(),
            metadata_opts: MetadataOptions::default(),
        })
    }

    /// Analyzes an audio file and extracts all properties
    pub fn analyze(&self, path: &Path) -> FormatResult<AudioProperties> {
        // Verify file exists
        if !path.exists() {
            return Err(FormatError::file_not_found(path.to_path_buf()));
        }

        // Get file size
        let file_size = std::fs::metadata(path)
            .map_err(|e| FormatError::read_error(path.to_path_buf(), e.to_string()))?
            .len();

        // Detect format from extension
        let format = AudioFormat::from_path(path).ok_or_else(|| {
            FormatError::unsupported(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown"),
                path.to_path_buf(),
            )
        })?;

        // Open file
        let file = File::open(path)
            .map_err(|e| FormatError::read_error(path.to_path_buf(), e.to_string()))?;

        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Create format hint
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        // Probe the file
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &self.format_opts, &self.metadata_opts)
            .map_err(|e| FormatError::probe_error(path.to_path_buf(), format!("{:?}", e)))?;

        let format_reader = probed.format;

        // Get the default track
        let track = format_reader
            .default_track()
            .ok_or_else(|| FormatError::probe_error(path.to_path_buf(), "No audio tracks found"))?;

        let params = &track.codec_params;

        // Extract properties
        let sample_rate = params
            .sample_rate
            .ok_or_else(|| FormatError::codec_error("Missing sample rate"))?;

        let channels = params.channels.map(|ch| ch.count() as u8).unwrap_or(2);

        let bits_per_sample = params.bits_per_sample.unwrap_or(16) as u8;

        // Calculate duration
        let duration = if let (Some(n_frames), Some(tb)) = (params.n_frames, params.time_base) {
            let time = tb.calc_time(n_frames);
            Some(Duration::from_secs_f64(time.seconds as f64 + time.frac))
        } else {
            None
        };

        // Symphonia 0.5 doesn't provide bitrate in CodecParameters
        // We'd need to calculate it from file size and duration
        let bitrate: Option<u32> = None;

        // Can't determine VBR without bitrate info
        let is_variable_bitrate = false;

        // Create quality assessment
        let is_lossless = format.is_lossless();
        let is_uncompressed = format.is_uncompressed();

        let quality = AudioQuality::new(
            sample_rate,
            bits_per_sample,
            is_lossless,
            is_uncompressed,
            bitrate,
        );

        let quality_tier = quality.tier;

        // Create codec info
        let codec = CodecInfo::from_params(params, format);

        Ok(AudioProperties {
            format,
            sample_rate,
            channels,
            bits_per_sample,
            duration,
            bitrate,
            is_variable_bitrate,
            quality,
            quality_tier,
            codec,
            total_samples: params.n_frames,
            file_size,
        })
    }

    /// Quick format detection without full analysis
    pub fn detect_format(&self, path: &Path) -> FormatResult<AudioFormat> {
        AudioFormat::from_path(path).ok_or_else(|| {
            FormatError::unsupported(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown"),
                path.to_path_buf(),
            )
        })
    }
}

impl Default for AudioAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create AudioAnalyzer")
    }
}

// === Helper Functions ===

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = AudioAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(65)), "1:05");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1:01:05");
        assert_eq!(format_duration(Duration::from_secs(45)), "0:45");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 bytes");
        assert_eq!(format_size(2048), "2.00 KB");
        assert_eq!(format_size(5_242_880), "5.00 MB");
        assert_eq!(format_size(2_147_483_648), "2.00 GB");
    }

    #[test]
    fn test_codec_info_creation() {
        let format = AudioFormat::Flac;
        let params = CodecParameters::new();
        let info = CodecInfo::from_params(&params, format);

        assert_eq!(info.name, "FLAC");
        assert!(info.is_lossless);
    }
}
