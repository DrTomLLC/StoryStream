// FILE: src/lib.rs
// ============================================================================

mod engine;
mod equalizer;
mod playback;
mod speed;
mod state;
mod decoder;

use std::fmt;
use std::time::Duration;

pub use engine::{AudioEngine, EngineConfig};
pub use equalizer::{Equalizer, EqualizerBand, EqualizerPreset};
pub use playback::{PlaybackState, PlaybackStatus};
pub use speed::PlaybackSpeed;
pub use state::EngineState;

/// Result type for media engine operations
pub type EngineResult<T> = Result<T, EngineError>;

/// Errors that can occur in the media engine
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    /// No audio file is currently loaded
    NoAudioLoaded,
    /// Invalid playback speed
    InvalidSpeed(f32),
    /// Invalid volume level
    InvalidVolume(u8),
    /// Invalid seek position
    InvalidPosition(Duration),
    /// Audio format not supported
    UnsupportedFormat(String),
    /// Failed to decode audio
    DecodeError(String),
    /// Failed to initialize audio output
    OutputError(String),
    /// Engine is in invalid state for operation
    InvalidState(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::NoAudioLoaded => write!(f, "No audio file is currently loaded"),
            EngineError::InvalidSpeed(s) => write!(f, "Invalid playback speed: {}", s),
            EngineError::InvalidVolume(v) => write!(f, "Invalid volume: {}", v),
            EngineError::InvalidPosition(p) => write!(f, "Invalid position: {:?}", p),
            EngineError::UnsupportedFormat(fmt) => write!(f, "Unsupported format: {}", fmt),
            EngineError::DecodeError(msg) => write!(f, "Decode error: {}", msg),
            EngineError::OutputError(msg) => write!(f, "Output error: {}", msg),
            EngineError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl std::error::Error for EngineError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EngineError::NoAudioLoaded;
        assert!(err.to_string().contains("No audio"));

        let err = EngineError::InvalidSpeed(3.5);
        assert!(err.to_string().contains("3.5"));
    }

    #[test]
    fn test_all_exports_accessible() {
        let _ = EngineConfig::default();
        let _ = PlaybackSpeed::default();
        let _ = Equalizer::default();
    }
}