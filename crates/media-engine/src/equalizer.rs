//! Audio equalizer

/// Audio equalizer with multiple bands
#[derive(Debug, Clone)]
pub struct Equalizer {
    enabled: bool,
    bands: Vec<EqualizerBand>,
    preset: Option<EqualizerPreset>,
}

impl Equalizer {
    /// Creates a new equalizer with 10 bands
    pub fn new_10_band() -> Self {
        Self {
            enabled: false,
            bands: vec![
                EqualizerBand::new(32.0, 0.0),    // Sub-bass
                EqualizerBand::new(64.0, 0.0),    // Bass
                EqualizerBand::new(125.0, 0.0),   // Bass
                EqualizerBand::new(250.0, 0.0),   // Low midrange
                EqualizerBand::new(500.0, 0.0),   // Midrange
                EqualizerBand::new(1000.0, 0.0),  // Midrange
                EqualizerBand::new(2000.0, 0.0),  // Upper midrange
                EqualizerBand::new(4000.0, 0.0),  // Presence
                EqualizerBand::new(8000.0, 0.0),  // Brilliance
                EqualizerBand::new(16000.0, 0.0), // Air
            ],
            preset: None,
        }
    }

    /// Enables or disables the equalizer
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether the equalizer is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets the gain for a specific band
    pub fn set_band_gain(&mut self, index: usize, gain_db: f32) -> Result<(), String> {
        if index >= self.bands.len() {
            return Err(format!("Band index {} out of range", index));
        }

        if !(-12.0..=12.0).contains(&gain_db) {
            return Err(format!("Gain {} dB out of range [-12, 12]", gain_db));
        }

        self.bands[index].gain_db = gain_db;
        self.preset = None; // Clear preset when manually adjusting
        Ok(())
    }

    /// Gets the gain for a specific band
    pub fn band_gain(&self, index: usize) -> Option<f32> {
        self.bands.get(index).map(|b| b.gain_db)
    }

    /// Returns all bands
    pub fn bands(&self) -> &[EqualizerBand] {
        &self.bands
    }

    /// Applies a preset
    pub fn apply_preset(&mut self, preset: EqualizerPreset) {
        let gains = preset.gains();
        for (i, gain) in gains.iter().enumerate() {
            if let Some(band) = self.bands.get_mut(i) {
                band.gain_db = *gain;
            }
        }
        self.preset = Some(preset);
    }

    /// Returns the current preset, if any
    pub fn current_preset(&self) -> Option<EqualizerPreset> {
        self.preset
    }

    /// Resets all bands to 0 dB (flat response)
    pub fn reset(&mut self) {
        for band in &mut self.bands {
            band.gain_db = 0.0;
        }
        self.preset = None;
    }

    /// Returns the number of bands
    pub fn band_count(&self) -> usize {
        self.bands.len()
    }
}

impl Default for Equalizer {
    fn default() -> Self {
        Self::new_10_band()
    }
}

/// A single equalizer band
#[derive(Debug, Clone, Copy)]
pub struct EqualizerBand {
    /// Center frequency in Hz
    pub frequency_hz: f32,
    /// Gain adjustment in dB
    pub gain_db: f32,
}

impl EqualizerBand {
    /// Creates a new equalizer band
    pub fn new(frequency_hz: f32, gain_db: f32) -> Self {
        Self {
            frequency_hz,
            gain_db,
        }
    }
}

/// Predefined equalizer presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqualizerPreset {
    /// Flat response (all bands at 0 dB)
    Flat,
    /// Enhanced bass
    BassBoost,
    /// Enhanced treble
    TrebleBoost,
    /// Voice/speech optimization
    Voice,
    /// Classical music
    Classical,
    /// Rock music
    Rock,
    /// Pop music
    Pop,
    /// Jazz music
    Jazz,
}

impl EqualizerPreset {
    /// Returns the gain values for each band in this preset
    pub fn gains(&self) -> Vec<f32> {
        match self {
            Self::Flat => vec![0.0; 10],
            Self::BassBoost => vec![6.0, 5.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Self::TrebleBoost => vec![0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 4.0, 5.0, 6.0, 6.0],
            Self::Voice => vec![-3.0, -2.0, 0.0, 2.0, 4.0, 4.0, 3.0, 1.0, -1.0, -2.0],
            Self::Classical => vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -2.0, -2.0, -2.0, -3.0],
            Self::Rock => vec![5.0, 3.0, 1.0, -1.0, -2.0, -1.0, 1.0, 3.0, 4.0, 5.0],
            Self::Pop => vec![2.0, 3.0, 4.0, 3.0, 0.0, -1.0, -1.0, 0.0, 2.0, 3.0],
            Self::Jazz => vec![3.0, 2.0, 0.0, 1.0, 2.0, 2.0, 1.0, 0.0, 2.0, 3.0],
        }
    }

    /// Returns all available presets
    pub fn all() -> Vec<Self> {
        vec![
            Self::Flat,
            Self::BassBoost,
            Self::TrebleBoost,
            Self::Voice,
            Self::Classical,
            Self::Rock,
            Self::Pop,
            Self::Jazz,
        ]
    }
}

impl std::fmt::Display for EqualizerPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flat => write!(f, "Flat"),
            Self::BassBoost => write!(f, "Bass Boost"),
            Self::TrebleBoost => write!(f, "Treble Boost"),
            Self::Voice => write!(f, "Voice"),
            Self::Classical => write!(f, "Classical"),
            Self::Rock => write!(f, "Rock"),
            Self::Pop => write!(f, "Pop"),
            Self::Jazz => write!(f, "Jazz"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equalizer_creation() {
        let eq = Equalizer::new_10_band();
        assert_eq!(eq.band_count(), 10);
        assert!(!eq.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let mut eq = Equalizer::new_10_band();
        eq.set_enabled(true);
        assert!(eq.is_enabled());
        eq.set_enabled(false);
        assert!(!eq.is_enabled());
    }

    #[test]
    fn test_set_band_gain() {
        let mut eq = Equalizer::new_10_band();
        assert!(eq.set_band_gain(0, 5.0).is_ok());
        assert_eq!(eq.band_gain(0), Some(5.0));
    }

    #[test]
    fn test_invalid_band_index() {
        let mut eq = Equalizer::new_10_band();
        assert!(eq.set_band_gain(100, 5.0).is_err());
    }

    #[test]
    fn test_invalid_gain() {
        let mut eq = Equalizer::new_10_band();
        assert!(eq.set_band_gain(0, 15.0).is_err());
        assert!(eq.set_band_gain(0, -15.0).is_err());
    }

    #[test]
    fn test_preset_application() {
        let mut eq = Equalizer::new_10_band();
        eq.apply_preset(EqualizerPreset::BassBoost);

        assert_eq!(eq.current_preset(), Some(EqualizerPreset::BassBoost));
        assert!(eq.band_gain(0).unwrap() > 0.0); // Bass boosted
    }

    #[test]
    fn test_reset() {
        let mut eq = Equalizer::new_10_band();
        eq.apply_preset(EqualizerPreset::BassBoost);
        eq.reset();

        for i in 0..eq.band_count() {
            assert_eq!(eq.band_gain(i), Some(0.0));
        }
        assert_eq!(eq.current_preset(), None);
    }

    #[test]
    fn test_manual_adjustment_clears_preset() {
        let mut eq = Equalizer::new_10_band();
        eq.apply_preset(EqualizerPreset::BassBoost);
        eq.set_band_gain(0, 3.0).unwrap();

        assert_eq!(eq.current_preset(), None);
    }

    #[test]
    fn test_all_presets_have_correct_length() {
        for preset in EqualizerPreset::all() {
            assert_eq!(preset.gains().len(), 10);
        }
    }
}