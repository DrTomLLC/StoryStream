use std::time::Duration;

/// Represents the current playback status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    /// Currently playing audio
    Playing,
    /// Playback is paused
    Paused,
    /// Playback is stopped
    Stopped,
}

/// Tracks the current playback state including position and status
#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub position: Duration,
    pub duration: Option<Duration>,
}

impl PlaybackState {
    /// Creates a new playback state in stopped position
    pub fn new() -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            position: Duration::from_secs(0),
            duration: None,
        }
    }

    /// Creates a playing state with given position and duration
    pub fn playing(position: Duration, duration: Option<Duration>) -> Self {
        Self {
            status: PlaybackStatus::Playing,
            position,
            duration,
        }
    }

    /// Creates a paused state with given position and duration
    pub fn paused(position: Duration, duration: Option<Duration>) -> Self {
        Self {
            status: PlaybackStatus::Paused,
            position,
            duration,
        }
    }

    /// Creates a stopped state
    pub fn stopped() -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            position: Duration::from_secs(0),
            duration: None,
        }
    }

    /// Returns true if currently playing
    pub fn is_playing(&self) -> bool {
        self.status == PlaybackStatus::Playing
    }

    /// Returns true if paused
    pub fn is_paused(&self) -> bool {
        self.status == PlaybackStatus::Paused
    }

    /// Returns true if stopped
    pub fn is_stopped(&self) -> bool {
        self.status == PlaybackStatus::Stopped
    }

    /// Returns the current playback position
    pub fn position(&self) -> Duration {
        self.position
    }

    /// Returns the total duration if known
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// Returns progress as a percentage (0.0 to 100.0)
    pub fn progress_percentage(&self) -> Option<f32> {
        self.duration.map(|dur| {
            if dur.as_secs() == 0 {
                0.0
            } else {
                (self.position.as_secs_f32() / dur.as_secs_f32()) * 100.0
            }
        })
    }

    /// Updates the position
    pub fn set_position(&mut self, position: Duration) {
        self.position = position;
    }

    /// Updates the status
    pub fn set_status(&mut self, status: PlaybackStatus) {
        self.status = status;
    }

    /// Updates the duration
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = Some(duration);
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
        assert!(state.is_stopped());
        assert_eq!(state.position(), Duration::from_secs(0));
        assert_eq!(state.duration(), None);
    }

    #[test]
    fn test_is_playing() {
        let state = PlaybackState::playing(Duration::from_secs(10), Some(Duration::from_secs(100)));
        assert!(state.is_playing());
        assert!(!state.is_paused());
        assert!(!state.is_stopped());
    }

    #[test]
    fn test_is_paused() {
        let state = PlaybackState::paused(Duration::from_secs(10), Some(Duration::from_secs(100)));
        assert!(!state.is_playing());
        assert!(state.is_paused());
        assert!(!state.is_stopped());
    }

    #[test]
    fn test_is_stopped() {
        let state = PlaybackState::stopped();
        assert!(!state.is_playing());
        assert!(!state.is_paused());
        assert!(state.is_stopped());
    }

    #[test]
    fn test_progress_percentage() {
        let state = PlaybackState::playing(Duration::from_secs(25), Some(Duration::from_secs(100)));
        let progress = state.progress_percentage().unwrap();
        assert!((progress - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_progress_zero_duration() {
        let state = PlaybackState::playing(Duration::from_secs(0), Some(Duration::from_secs(0)));
        assert_eq!(state.progress_percentage(), Some(0.0));
    }

    #[test]
    fn test_progress_no_duration() {
        let state = PlaybackState::playing(Duration::from_secs(25), None);
        assert_eq!(state.progress_percentage(), None);
    }

    #[test]
    fn test_set_position() {
        let mut state = PlaybackState::new();
        state.set_position(Duration::from_secs(42));
        assert_eq!(state.position(), Duration::from_secs(42));
    }

    #[test]
    fn test_set_status() {
        let mut state = PlaybackState::new();
        state.set_status(PlaybackStatus::Playing);
        assert!(state.is_playing());
    }

    #[test]
    fn test_set_duration() {
        let mut state = PlaybackState::new();
        state.set_duration(Duration::from_secs(300));
        assert_eq!(state.duration(), Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_playing_constructor() {
        let state = PlaybackState::playing(Duration::from_secs(10), Some(Duration::from_secs(100)));
        assert!(state.is_playing());
        assert_eq!(state.position(), Duration::from_secs(10));
        assert_eq!(state.duration(), Some(Duration::from_secs(100)));
    }

    #[test]
    fn test_paused_constructor() {
        let state = PlaybackState::paused(Duration::from_secs(10), Some(Duration::from_secs(100)));
        assert!(state.is_paused());
        assert_eq!(state.position(), Duration::from_secs(10));
        assert_eq!(state.duration(), Some(Duration::from_secs(100)));
    }

    #[test]
    fn test_stopped_constructor() {
        let state = PlaybackState::stopped();
        assert!(state.is_stopped());
        assert_eq!(state.position(), Duration::from_secs(0));
        assert_eq!(state.duration(), None);
    }
}
