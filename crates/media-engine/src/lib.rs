// FILE: crates/media-engine/src/lib.rs

mod decoder;
mod engine;
mod equalizer;
mod error;
mod output;
mod playback;
mod playback_thread;
mod speed;
mod state;

pub use engine::{AudioEngine, EngineConfig};
pub use equalizer::{Equalizer, EqualizerBand, EqualizerPreset};
pub use error::{EngineError, EngineResult};
pub use playback::{PlaybackState, PlaybackStatus};
pub use speed::PlaybackSpeed;
pub use state::EngineState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_exports_accessible() {
        let _ = AudioEngine::new();
        let _ = Equalizer::default();
        let _ = PlaybackSpeed::default();
    }

    #[test]
    fn test_error_display() {
        let err = EngineError::DecodeError("test".to_string());
        assert!(format!("{}", err).contains("test"));
    }
}