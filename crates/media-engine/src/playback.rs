// FILE: src/playback.rs
// ============================================================================

use std::time::Duration;
/// Playback status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

/// Playback state information
#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub position: Duration,
    pub duration: Option<Duration>,
}

impl PlaybackState {
    pub fn new() -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            position: Duration::ZERO,
            duration: None,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.status == PlaybackStatus::Playing
    }

    pub fn is_paused(&self) -> bool {
        self.status == PlaybackStatus::Paused
    }

    pub fn is_stopped(&self) -> bool {
        self.status == PlaybackStatus::Stopped
    }

    pub fn progress_percentage(&self) -> Option<f32> {
        self.duration.map(|d| {
            if d.as_secs() == 0 {
                0.0
            } else {
                (self.position.as_secs_f32() / d.as_secs_f32()) * 100.0
            }
        })
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod playback_tests {
    use super::*;

    #[test]
    fn test_playback_state_new() {
        let state = PlaybackState::new();
        assert_eq!(state.status, PlaybackStatus::Stopped);
        assert_eq!(state.position, Duration::ZERO);
    }

    #[test]
    fn test_is_playing() {
        let mut state = PlaybackState::new();
        assert!(!state.is_playing());
        state.status = PlaybackStatus::Playing;
        assert!(state.is_playing());
    }

    #[test]
    fn test_progress_percentage() {
        let mut state = PlaybackState::new();
        state.duration = Some(Duration::from_secs(100));
        state.position = Duration::from_secs(50);
        assert_eq!(state.progress_percentage(), Some(50.0));
    }

    #[test]
    fn test_progress_zero_duration() {
        let mut state = PlaybackState::new();
        state.duration = Some(Duration::ZERO);
        assert_eq!(state.progress_percentage(), Some(0.0));
    }
}
