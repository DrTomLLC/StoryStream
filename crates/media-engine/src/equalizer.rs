// crates/media-engine/src/equalizer.rs
use dasp::signal::Signal;

pub struct ParametricEQ {
    bands: Vec<BiquadFilter>,
}

pub struct BiquadFilter {
    frequency: f32,    // Center frequency
    gain: f32,         // dB gain
    q: f32,            // Quality factor (bandwidth)
    filter_type: FilterType,
}

pub enum FilterType {
    LowShelf,
    HighShelf,
    Peak,
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

impl ParametricEQ {
    /// Professional 10-band parametric EQ
    pub fn new_10_band(sample_rate: u32) -> Self {
        let bands = vec![
            // Sub-bass
            BiquadFilter::new(32.0, 0.0, 0.7, FilterType::LowShelf),
            // Bass
            BiquadFilter::new(64.0, 0.0, 1.0, FilterType::Peak),
            BiquadFilter::new(125.0, 0.0, 1.0, FilterType::Peak),
            // Midrange
            BiquadFilter::new(250.0, 0.0, 1.0, FilterType::Peak),
            BiquadFilter::new(500.0, 0.0, 1.0, FilterType::Peak),
            BiquadFilter::new(1000.0, 0.0, 1.0, FilterType::Peak),
            BiquadFilter::new(2000.0, 0.0, 1.0, FilterType::Peak),
            // Treble
            BiquadFilter::new(4000.0, 0.0, 1.0, FilterType::Peak),
            BiquadFilter::new(8000.0, 0.0, 1.0, FilterType::Peak),
            BiquadFilter::new(16000.0, 0.0, 0.7, FilterType::HighShelf),
        ];

        Self { bands }
    }

    /// Process audio through EQ chain
    pub fn process(&mut self, samples: &mut [f32]) {
        for band in &mut self.bands {
            band.process(samples);
        }
    }
}