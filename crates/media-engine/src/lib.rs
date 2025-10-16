// crates/media-engine/src/lib.rs
// REPLACE THE ENTIRE FILE WITH THIS:

//! Media Engine
//!
//! Core audio playback engine with support for:
//! - Multiple audio formats (MP3, FLAC, WAV, OGG, etc.)
//! - Variable playback speed with optional pitch correction
//! - 10-band equalizer with presets
//! - Chapter navigation
//! - Gapless playback
//! - Audio device selection
//! - Bookmark management

pub mod audio_device;
pub mod bookmarks;
pub mod chapters;
pub mod decoder;
pub mod engine;
pub mod equalizer;
pub mod error;
pub mod output;
pub mod playback;
pub mod playback_thread;
pub mod speed;
pub mod state;

// Re-export main types for convenience
pub use audio_device::{AudioDeviceInfo, AudioDeviceManager};
pub use bookmarks::{Bookmark, BookmarkManager, BookmarkType};
pub use chapters::{ChapterList, ChapterMarker};
pub use decoder::AudioDecoder;
pub use engine::{EngineConfig, MediaEngine};
pub use equalizer::{Equalizer, EqualizerBand, EqualizerPreset};
pub use error::{EngineError, EngineResult};
pub use output::{AudioOutput, AudioOutputConfig};
pub use playback::{PlaybackState, PlaybackStatus};
pub use speed::{Speed, SpeedProcessor};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_exports_accessible() {
        let _config = EngineConfig::default();
        let _speed = Speed::default();
        let _eq = Equalizer::default();
        let _output_config = AudioOutputConfig::default();
    }

    #[test]
    fn test_error_display() {
        let err = EngineError::DecodeError("test".into());
        assert!(!format!("{}", err).is_empty());
    }
}
