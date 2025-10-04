use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Current state of media playback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
    Buffering,
}

/// Represents a chapter in an audiobook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub index: usize,
    pub title: String,
    pub start_time: Duration,
    pub end_time: Duration,
}

impl Chapter {
    pub fn new(index: usize, title: String, start_time: Duration, end_time: Duration) -> Self {
        Self {
            index,
            title,
            start_time,
            end_time,
        }
    }

    pub fn duration(&self) -> Duration {
        self.end_time.saturating_sub(self.start_time)
    }

    pub fn contains(&self, position: Duration) -> bool {
        position >= self.start_time && position < self.end_time
    }
}

/// Complete playback state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub position: Duration,
    pub duration: Duration,
    pub speed: f32,
    pub volume: f32,
    pub book_id: Option<i64>,
    pub current_chapter: Option<usize>,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            position: Duration::ZERO,
            duration: Duration::ZERO,
            speed: 1.0,
            volume: 1.0,
            book_id: None,
            current_chapter: None,
        }
    }
}

impl PlaybackState {
    pub fn progress(&self) -> f32 {
        if self.duration.as_secs() == 0 {
            0.0
        } else {
            self.position.as_secs_f32() / self.duration.as_secs_f32()
        }
    }

    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.position)
    }

    pub fn is_playing(&self) -> bool {
        self.status == PlaybackStatus::Playing
    }

    pub fn is_paused(&self) -> bool {
        self.status == PlaybackStatus::Paused
    }
}

/// Events emitted by the media engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaEvent {
    StateChanged(PlaybackState),
    PositionChanged(Duration),
    ChapterChanged(usize),
    PlaybackEnded,
    Error(String),
}