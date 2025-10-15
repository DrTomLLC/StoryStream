//! Media Engine - Audio playback engine for StoryStream

mod chapters;
mod decoder;
mod engine;
mod equalizer;
mod error;
mod output;
mod playback;
pub(crate) mod playback_thread;
mod speed;
mod state;

pub use chapters::{ChapterList, ChapterMarker};
pub use decoder::AudioDecoder;
pub use engine::MediaEngine;
pub use equalizer::{Equalizer, EqualizerPreset};
pub use error::{EngineError, EngineResult};
pub use output::AudioOutput;
pub use speed::{Speed, SpeedProcessor};
pub use state::{EngineState, PlaybackState};
pub use storystream_core::PlaybackSpeed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

pub type Result<T> = std::result::Result<T, EngineError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_exports_accessible() {
        // Just test that types are accessible
        let _ = PlaybackStatus::Stopped;
        let _ = ChapterList::new();
    }

    #[test]
    fn test_error_display() {
        let error = EngineError::InvalidSpeed(5.0);
        assert!(format!("{}", error).contains("5"));
    }
}