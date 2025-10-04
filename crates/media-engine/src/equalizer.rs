// FILE: src/equalizer.rs
// ============================================================================

/// 10-band equalizer
#[derive(Debug, Clone, PartialEq)]
pub struct Equalizer {
    bands: [EqualizerBand; 10],
    enabled: bool,
}

/// Single equalizer band
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EqualizerBand {
    pub frequency: u32,
    pub gain: f32, // -12.0 to +12.0 dB
}

impl Equalizer {
    /// Frequency bands (Hz)
    pub const BANDS: [u32; 10] = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

    pub fn new() -> Self {
        Self {
            bands: [
                EqualizerBand {
                    frequency: 32,
                    gain: 0.0,
                },
                EqualizerBand {
                    frequency: 64,
                    gain: 0.0,
                },
                EqualizerBand {
                    frequency: 125,
                    gain: 0.0,
                },
                EqualizerBand {
                    frequency: 250,
                    gain: 0.0,
                },
                EqualizerBand {
                    frequency: 500,
                    gain: 0.0,
                },
                EqualizerBand {
                    frequency: 1000,
                    gain: 0.0,
                },
                EqualizerBand {
                    frequency: 2000,
                    gain: 0.0,
                },
                EqualizerBand {
                    frequency: 4000,
                    gain: 0.0,
                },
                EqualizerBand {
                    frequency: 8000,
                    gain: 0.0,
                },
                EqualizerBand {
                    frequency: 16000,
                    gain: 0.0,
                },
            ],
            enabled: false,
        }
    }

    pub fn set_gain(&mut self, band_index: usize, gain: f32) -> Result<(), String> {
        if band_index >= 10 {
            return Err(format!("Invalid band index: {}", band_index));
        }
        if !(-12.0..=12.0).contains(&gain) {
            return Err(format!("Gain out of range: {}", gain));
        }
        self.bands[band_index].gain = gain;
        Ok(())
    }

    pub fn get_gain(&self, band_index: usize) -> Option<f32> {
        self.bands.get(band_index).map(|b| b.gain)
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn apply_preset(&mut self, preset: EqualizerPreset) {
        let gains = preset.gains();
        for (i, &gain) in gains.iter().enumerate() {
            self.bands[i].gain = gain;
        }
        self.enabled = true;
    }

    pub fn reset(&mut self) {
        for band in &mut self.bands {
            band.gain = 0.0;
        }
    }
}

impl Default for Equalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Equalizer presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqualizerPreset {
    Flat,
    BassBoost,
    VoiceBoost,
    Treble,
}

impl EqualizerPreset {
    fn gains(&self) -> [f32; 10] {
        match self {
            EqualizerPreset::Flat => [0.0; 10],
            EqualizerPreset::BassBoost => [6.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            EqualizerPreset::VoiceBoost => [0.0, 0.0, 0.0, 3.0, 4.0, 3.0, 0.0, 0.0, 0.0, 0.0],
            EqualizerPreset::Treble => [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 4.0, 6.0, 6.0],
        }
    }
}

#[cfg(test)]
mod equalizer_tests {
    use super::*;

    #[test]
    fn test_equalizer_new() {
        let eq = Equalizer::new();
        assert!(!eq.is_enabled());
        assert_eq!(eq.get_gain(0), Some(0.0));
    }

    #[test]
    fn test_set_gain() {
        let mut eq = Equalizer::new();
        assert!(eq.set_gain(0, 5.0).is_ok());
        assert_eq!(eq.get_gain(0), Some(5.0));
    }

    #[test]
    fn test_invalid_band() {
        let mut eq = Equalizer::new();
        assert!(eq.set_gain(10, 0.0).is_err());
    }

    #[test]
    fn test_invalid_gain() {
        let mut eq = Equalizer::new();
        assert!(eq.set_gain(0, 15.0).is_err());
        assert!(eq.set_gain(0, -15.0).is_err());
    }

    #[test]
    fn test_enable_disable() {
        let mut eq = Equalizer::new();
        assert!(!eq.is_enabled());
        eq.enable();
        assert!(eq.is_enabled());
        eq.disable();
        assert!(!eq.is_enabled());
    }

    #[test]
    fn test_preset_flat() {
        let mut eq = Equalizer::new();
        eq.apply_preset(EqualizerPreset::Flat);
        assert!(eq.is_enabled());
        for i in 0..10 {
            assert_eq!(eq.get_gain(i), Some(0.0));
        }
    }

    #[test]
    fn test_preset_bass_boost() {
        let mut eq = Equalizer::new();
        eq.apply_preset(EqualizerPreset::BassBoost);
        assert!(eq.get_gain(0).unwrap() > 0.0);
        assert!(eq.get_gain(9) == Some(0.0));
    }

    #[test]
    fn test_reset() {
        let mut eq = Equalizer::new();
        eq.set_gain(0, 5.0).unwrap();
        eq.reset();
        assert_eq!(eq.get_gain(0), Some(0.0));
    }
}