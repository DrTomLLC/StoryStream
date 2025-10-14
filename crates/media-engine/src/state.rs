// FILE: src/state.rs
// ============================================================================

use crate::{Equalizer, PlaybackSpeed, PlaybackStatus};
use std::path::PathBuf;
use std::time::Duration;

/// Internal engine state
#[derive(Debug, Clone)]
pub struct EngineState {
    pub status: PlaybackStatus,
    pub current_file: Option<PathBuf>,
    pub position: Duration,
    pub duration: Option<Duration>,
    pub volume: u8,
    pub speed: PlaybackSpeed,
    pub equalizer: Equalizer,
    pub chapters: Vec<Chapter>,
    pub current_chapter: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub title: String,
    pub start: Duration,
    pub end: Duration,
}

#[cfg(test)]
mod state_tests {
    use super::*;

    #[test]
    fn test_chapter_creation() {
        let chapter = Chapter {
            title: "Chapter 1".to_string(),
            start: Duration::ZERO,
            end: Duration::from_secs(600),
        };
        assert_eq!(chapter.title, "Chapter 1");
    }
}