//! Engine state management

use crate::{Equalizer, PlaybackStatus};
use storystream_core::PlaybackSpeed;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Chapter {
    pub title: String,
    pub start_time: Duration,
    pub end_time: Duration,
}

impl Chapter {
    pub fn new(title: String, start_time: Duration, end_time: Duration) -> Self {
        Self {
            title,
            start_time,
            end_time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaybackState {
    is_playing: bool,
    position: f64,
    duration: f64,
}

impl PlaybackState {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            position: 0.0,
            duration: 0.0,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.is_playing = playing;
    }

    pub fn position(&self) -> f64 {
        self.position
    }

    pub fn set_position(&mut self, position: f64) {
        self.position = position;
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn set_duration(&mut self, duration: f64) {
        self.duration = duration;
    }

    pub fn progress_percentage(&self) -> f32 {
        if self.duration == 0.0 {
            return 0.0;
        }
        ((self.position / self.duration) * 100.0) as f32
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct EngineState {
    status: PlaybackStatus,
    position: f64,
    volume: f32,
    speed: PlaybackSpeed,
    equalizer: Equalizer,
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            position: 0.0,
            volume: 1.0,
            speed: PlaybackSpeed::new(1.0).unwrap(),
            equalizer: Equalizer::new_10_band(),
        }
    }

    pub fn status(&self) -> PlaybackStatus {
        self.status
    }

    pub fn set_status(&mut self, status: PlaybackStatus) {
        self.status = status;
    }

    pub fn position(&self) -> f64 {
        self.position
    }

    pub fn set_position(&mut self, position: f64) {
        self.position = position;
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    pub fn speed(&self) -> PlaybackSpeed {
        self.speed
    }

    pub fn set_speed(&mut self, speed: PlaybackSpeed) {
        self.speed = speed;
    }

    pub fn equalizer(&self) -> &Equalizer {
        &self.equalizer
    }

    pub fn set_equalizer(&mut self, equalizer: Equalizer) {
        self.equalizer = equalizer;
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;

    #[test]
    fn test_chapter_creation() {
        let chapter = Chapter::new(
            "Chapter 1".to_string(),
            Duration::from_secs(0),
            Duration::from_secs(60),
        );
        assert_eq!(chapter.title, "Chapter 1");
    }

    #[test]
    fn test_playback_state_new() {
        let state = PlaybackState::new();
        assert!(!state.is_playing());
        assert_eq!(state.position(), 0.0);
    }

    #[test]
    fn test_is_playing() {
        let mut state = PlaybackState::new();
        assert!(!state.is_playing());
        state.set_playing(true);
        assert!(state.is_playing());
    }

    #[test]
    fn test_progress_percentage() {
        let mut state = PlaybackState::new();
        state.set_duration(100.0);
        state.set_position(50.0);
        assert_eq!(state.progress_percentage(), 50.0);
    }

    #[test]
    fn test_progress_zero_duration() {
        let state = PlaybackState::new();
        assert_eq!(state.progress_percentage(), 0.0);
    }
}