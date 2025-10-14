//! Audio quality classification and analysis

use std::fmt;

/// Audio quality tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QualityTier {
    /// Low quality (< 128 kbps lossy or heavily compressed)
    Low,
    /// Standard quality (128-192 kbps lossy)
    Standard,
    /// High quality (192-320 kbps lossy)
    High,
    /// CD quality (16-bit/44.1kHz lossless or uncompressed)
    CD,
    /// DVD quality (16-bit/48kHz)
    DVD,
    /// Hi-Res (24-bit/96kHz)
    HiRes96,
    /// Ultra Hi-Res (24-bit/192kHz or higher)
    HiRes192,
    /// Studio quality (32-bit float)
    Studio,
}

impl QualityTier {
    /// Determines quality tier from audio properties
    pub fn from_properties(
        sample_rate: u32,
        bits_per_sample: u8,
        is_lossless: bool,
        bitrate: Option<u32>,
    ) -> Self {
        if is_lossless {
            // Lossless or uncompressed
            match (sample_rate, bits_per_sample) {
                (_, 32) => Self::Studio,
                (192_000..=u32::MAX, 24) => Self::HiRes192,
                (96_000..=191_999, 24) => Self::HiRes96,
                (48_000..=95_999, _) => Self::DVD,
                (44_100..=47_999, _) => Self::CD,
                _ => Self::Standard,
            }
        } else if let Some(bitrate) = bitrate {
            // Lossy format - classify by bitrate
            match bitrate {
                320_000..=u32::MAX => Self::High,
                192_000..=319_999 => Self::High,
                128_000..=191_999 => Self::Standard,
                _ => Self::Low,
            }
        } else {
            // Unknown bitrate, use conservative estimate
            Self::Standard
        }
    }

    /// Returns the minimum sample rate for this tier
    pub fn min_sample_rate(&self) -> u32 {
        match self {
            Self::Low | Self::Standard => 22_050,
            Self::High => 32_000,
            Self::CD => 44_100,
            Self::DVD => 48_000,
            Self::HiRes96 => 96_000,
            Self::HiRes192 | Self::Studio => 192_000,
        }
    }

    /// Returns the minimum bit depth for this tier
    pub fn min_bit_depth(&self) -> u8 {
        match self {
            Self::Low | Self::Standard | Self::High => 16,
            Self::CD | Self::DVD => 16,
            Self::HiRes96 | Self::HiRes192 => 24,
            Self::Studio => 32,
        }
    }

    /// Returns true if this tier represents audiophile quality
    pub fn is_audiophile(&self) -> bool {
        matches!(
            self,
            Self::HiRes96 | Self::HiRes192 | Self::Studio
        )
    }

    /// Returns true if this is considered high fidelity
    pub fn is_high_fidelity(&self) -> bool {
        *self >= Self::CD
    }

    /// Returns a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Low => "Low quality (suitable for voice/podcasts)",
            Self::Standard => "Standard quality (acceptable for most content)",
            Self::High => "High quality (near-transparent lossy)",
            Self::CD => "CD quality (16-bit/44.1kHz lossless)",
            Self::DVD => "DVD quality (16-bit/48kHz)",
            Self::HiRes96 => "Hi-Res audio (24-bit/96kHz)",
            Self::HiRes192 => "Ultra Hi-Res audio (24-bit/192kHz+)",
            Self::Studio => "Studio master quality (32-bit float)",
        }
    }
}

impl fmt::Display for QualityTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Standard => write!(f, "Standard"),
            Self::High => write!(f, "High"),
            Self::CD => write!(f, "CD Quality"),
            Self::DVD => write!(f, "DVD Quality"),
            Self::HiRes96 => write!(f, "Hi-Res 96kHz"),
            Self::HiRes192 => write!(f, "Hi-Res 192kHz"),
            Self::Studio => write!(f, "Studio"),
        }
    }
}

/// Complete audio quality assessment
#[derive(Debug, Clone, PartialEq)]
pub struct AudioQuality {
    /// Overall quality tier
    pub tier: QualityTier,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Bits per sample
    pub bits_per_sample: u8,
    /// Bitrate in bps (for lossy formats)
    pub bitrate: Option<u32>,
    /// Is lossless compression
    pub is_lossless: bool,
    /// Is uncompressed
    pub is_uncompressed: bool,
    /// Dynamic range in dB (if calculable)
    pub dynamic_range_db: Option<f32>,
}

impl AudioQuality {
    /// Creates a new audio quality assessment
    pub fn new(
        sample_rate: u32,
        bits_per_sample: u8,
        is_lossless: bool,
        is_uncompressed: bool,
        bitrate: Option<u32>,
    ) -> Self {
        let tier = QualityTier::from_properties(
            sample_rate,
            bits_per_sample,
            is_lossless || is_uncompressed,
            bitrate,
        );

        Self {
            tier,
            sample_rate,
            bits_per_sample,
            bitrate,
            is_lossless,
            is_uncompressed,
            dynamic_range_db: None,
        }
    }

    /// Sets the dynamic range
    pub fn with_dynamic_range(mut self, db: f32) -> Self {
        self.dynamic_range_db = Some(db);
        self
    }

    /// Returns a quality score (0-100)
    pub fn score(&self) -> u8 {
        let tier_score = match self.tier {
            QualityTier::Low => 30,
            QualityTier::Standard => 50,
            QualityTier::High => 70,
            QualityTier::CD => 80,
            QualityTier::DVD => 85,
            QualityTier::HiRes96 => 92,
            QualityTier::HiRes192 => 98,
            QualityTier::Studio => 100,
        };

        // Bonus for lossless
        let lossless_bonus = if self.is_lossless || self.is_uncompressed {
            5
        } else {
            0
        };

        (tier_score + lossless_bonus).min(100)
    }

    /// Returns a detailed quality report
    pub fn report(&self) -> String {
        let mut report = format!(
            "Quality: {} ({})\n\
             Sample Rate: {} Hz\n\
             Bit Depth: {} bits\n\
             Compression: {}",
            self.tier,
            self.tier.description(),
            self.sample_rate,
            self.bits_per_sample,
            if self.is_uncompressed {
                "None (Uncompressed)"
            } else if self.is_lossless {
                "Lossless"
            } else {
                "Lossy"
            }
        );

        if let Some(bitrate) = self.bitrate {
            report.push_str(&format!("\nBitrate: {} kbps", bitrate / 1000));
        }

        if let Some(dr) = self.dynamic_range_db {
            report.push_str(&format!("\nDynamic Range: {:.1} dB", dr));
        }

        report.push_str(&format!("\nQuality Score: {}/100", self.score()));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_tier_ordering() {
        assert!(QualityTier::Low < QualityTier::Standard);
        assert!(QualityTier::Standard < QualityTier::High);
        assert!(QualityTier::High < QualityTier::CD);
        assert!(QualityTier::CD < QualityTier::HiRes96);
    }

    #[test]
    fn test_cd_quality() {
        let tier = QualityTier::from_properties(44_100, 16, true, None);
        assert_eq!(tier, QualityTier::CD);
        assert!(tier.is_high_fidelity());
    }

    #[test]
    fn test_hires_quality() {
        let tier = QualityTier::from_properties(96_000, 24, true, None);
        assert_eq!(tier, QualityTier::HiRes96);
        assert!(tier.is_audiophile());
    }

    #[test]
    fn test_lossy_quality() {
        let tier = QualityTier::from_properties(44_100, 16, false, Some(320_000));
        assert_eq!(tier, QualityTier::High);

        let tier = QualityTier::from_properties(44_100, 16, false, Some(128_000));
        assert_eq!(tier, QualityTier::Standard);

        let tier = QualityTier::from_properties(44_100, 16, false, Some(96_000));
        assert_eq!(tier, QualityTier::Low);
    }

    #[test]
    fn test_audio_quality() {
        let quality = AudioQuality::new(44_100, 16, true, false, None);
        assert_eq!(quality.tier, QualityTier::CD);
        assert!(quality.score() >= 80);
    }

    #[test]
    fn test_quality_score() {
        let q1 = AudioQuality::new(44_100, 16, false, false, Some(320_000));
        let q2 = AudioQuality::new(44_100, 16, true, false, None);
        let q3 = AudioQuality::new(192_000, 24, true, false, None);

        assert!(q2.score() > q1.score()); // Lossless > lossy
        assert!(q3.score() > q2.score()); // Hi-res > CD
    }

    #[test]
    fn test_quality_report() {
        let quality = AudioQuality::new(96_000, 24, true, false, None)
            .with_dynamic_range(14.2);

        let report = quality.report();
        assert!(report.contains("Hi-Res"));
        assert!(report.contains("96000"));
        assert!(report.contains("24"));
        assert!(report.contains("Lossless"));
        assert!(report.contains("14.2"));
    }
}