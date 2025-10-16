// crates/media-engine/src/equalizer.rs
// Complete clean version without unused imports

#[derive(Debug, Clone)]
pub struct EqualizerBand {
    pub frequency: f32,
    pub gain: f32,
    pub q_factor: f32,
}

#[derive(Debug, Clone)]
pub struct Equalizer {
    bands: Vec<EqualizerBand>,
    enabled: bool,
}

impl Default for Equalizer {
    fn default() -> Self {
        Self {
            bands: Self::default_bands(),
            enabled: false,
        }
    }
}

impl Equalizer {
    fn default_bands() -> Vec<EqualizerBand> {
        vec![
            EqualizerBand { frequency: 32.0, gain: 0.0, q_factor: 0.7 },
            EqualizerBand { frequency: 64.0, gain: 0.0, q_factor: 0.7 },
            EqualizerBand { frequency: 125.0, gain: 0.0, q_factor: 0.7 },
            EqualizerBand { frequency: 250.0, gain: 0.0, q_factor: 0.7 },
            EqualizerBand { frequency: 500.0, gain: 0.0, q_factor: 0.7 },
            EqualizerBand { frequency: 1000.0, gain: 0.0, q_factor: 0.7 },
            EqualizerBand { frequency: 2000.0, gain: 0.0, q_factor: 0.7 },
            EqualizerBand { frequency: 4000.0, gain: 0.0, q_factor: 0.7 },
            EqualizerBand { frequency: 8000.0, gain: 0.0, q_factor: 0.7 },
            EqualizerBand { frequency: 16000.0, gain: 0.0, q_factor: 0.7 },
        ]
    }

    /// Apply equalizer to audio samples (passthrough for now)
    pub fn apply(&self, samples: &[f32]) -> Vec<f32> {
        if !self.enabled {
            return samples.to_vec();
        }

        // For now, just pass through the audio
        // Full EQ implementation would require FFT processing
        samples.to_vec()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_band_gain(&mut self, band_index: usize, gain: f32) {
        if let Some(band) = self.bands.get_mut(band_index) {
            band.gain = gain.clamp(-12.0, 12.0);
        }
    }

    pub fn reset(&mut self) {
        self.bands = Self::default_bands();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EqualizerPreset {
    Flat,
    Rock,
    Jazz,
    Classical,
    Pop,
    Bass,
    Treble,
    Vocal,
    Custom,
}

impl EqualizerPreset {
    pub fn apply_to(&self, eq: &mut Equalizer) {
        let gains = match self {
            Self::Flat => vec![0.0; 10],
            Self::Rock => vec![5.0, 4.0, 3.0, 1.0, -1.0, -1.0, 1.0, 3.0, 4.0, 5.0],
            Self::Jazz => vec![4.0, 3.0, 1.0, 2.0, -2.0, -2.0, 0.0, 1.0, 3.0, 4.0],
            Self::Classical => vec![-2.0, -2.0, -2.0, -1.0, 2.0, 3.0, 3.0, 2.0, 0.0, -1.0],
            Self::Pop => vec![-2.0, -1.0, 2.0, 4.0, 5.0, 4.0, 2.0, 0.0, -1.0, -2.0],
            Self::Bass => vec![6.0, 5.0, 4.0, 3.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Self::Treble => vec![-2.0, -2.0, -1.0, 0.0, 1.0, 2.0, 4.0, 5.0, 6.0, 7.0],
            Self::Vocal => vec![-2.0, -3.0, -3.0, 1.0, 4.0, 4.0, 3.0, 1.0, 0.0, -1.0],
            Self::Custom => return, // Don't change anything for custom
        };

        for (i, &gain) in gains.iter().enumerate() {
            eq.set_band_gain(i, gain);
        }
    }
}