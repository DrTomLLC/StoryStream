//! Core media engine for audio playback with chapter support

use crate::chapters::{ChapterMarker, ChapterList};
use crate::playback_thread::{PlaybackCommand, PlaybackThread};
use crate::state::EngineState;
use crate::{EngineError, EngineResult, Equalizer, PlaybackStatus};
use storystream_core::PlaybackSpeed;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct MediaEngine {
    state: Arc<Mutex<EngineState>>,
    playback_thread: Option<PlaybackThread>,
    current_file: Option<PathBuf>,
    chapter_list: Arc<Mutex<ChapterList>>,
}

impl MediaEngine {
    pub fn new() -> EngineResult<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(EngineState::default())),
            playback_thread: None,
            current_file: None,
            chapter_list: Arc::new(Mutex::new(ChapterList::new())),
        })
    }

    pub fn with_config(_config: ()) -> EngineResult<Self> {
        Self::new()
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(EngineError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            )));
        }

        // Stop any existing playback
        if let Some(mut thread) = self.playback_thread.take() {
            thread.stop();
        }

        // Store the file path
        self.current_file = Some(path.to_path_buf());

        // Reset state
        let mut state = self.state.lock().unwrap();
        state.set_status(PlaybackStatus::Stopped);
        state.set_position(0.0);

        // Reset chapters
        let mut chapter_list = self.chapter_list.lock().unwrap();
        *chapter_list = ChapterList::new();

        Ok(())
    }

    /// Loads chapters for the current file
    pub fn load_chapters(&mut self, chapters: Vec<ChapterMarker>) -> EngineResult<()> {
        let mut chapter_list = self.chapter_list.lock().unwrap();
        *chapter_list = ChapterList::with_chapters(chapters);
        Ok(())
    }

    pub fn play(&mut self) -> EngineResult<()> {
        let path = self.current_file.as_ref().ok_or_else(|| {
            EngineError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file loaded",
            ))
        })?;

        // Start playback thread if not already running
        if self.playback_thread.is_none() {
            let thread = PlaybackThread::start(path)?;
            self.playback_thread = Some(thread);
        }

        // Send play command
        if let Some(thread) = &self.playback_thread {
            thread.send_command(PlaybackCommand::Play)?;
        }

        // Update state
        let mut state = self.state.lock().unwrap();
        state.set_status(PlaybackStatus::Playing);

        Ok(())
    }

    pub fn pause(&mut self) -> EngineResult<()> {
        if self.playback_thread.is_none() {
            return Err(EngineError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file loaded",
            )));
        }

        // Send pause command
        if let Some(thread) = &self.playback_thread {
            thread.send_command(PlaybackCommand::Pause)?;
        }

        // Update state
        let mut state = self.state.lock().unwrap();
        state.set_status(PlaybackStatus::Paused);

        Ok(())
    }

    pub fn stop(&mut self) -> EngineResult<()> {
        // Stop the playback thread
        if let Some(mut thread) = self.playback_thread.take() {
            thread.stop();
        }

        // Update state
        let mut state = self.state.lock().unwrap();
        state.set_status(PlaybackStatus::Stopped);
        state.set_position(0.0);

        Ok(())
    }

    pub fn seek(&mut self, position: f64) -> EngineResult<()> {
        if self.playback_thread.is_none() {
            return Err(EngineError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file loaded",
            )));
        }

        // Send seek command
        if let Some(thread) = &self.playback_thread {
            thread.send_command(PlaybackCommand::Seek(position))?;
        }

        // Update state and chapter position
        let mut state = self.state.lock().unwrap();
        state.set_position(position);

        let mut chapter_list = self.chapter_list.lock().unwrap();
        chapter_list.update_position(position);

        Ok(())
    }

    /// Seeks to the next chapter
    pub fn next_chapter(&mut self) -> EngineResult<()> {
        let mut chapter_list = self.chapter_list.lock().unwrap();

        if let Some(start_time) = chapter_list.go_to_next() {
            drop(chapter_list); // Release lock before seek
            self.seek(start_time)?;
            Ok(())
        } else {
            Err(EngineError::InvalidState("No next chapter".to_string()))
        }
    }

    /// Seeks to the previous chapter
    pub fn previous_chapter(&mut self) -> EngineResult<()> {
        let mut chapter_list = self.chapter_list.lock().unwrap();

        if let Some(start_time) = chapter_list.go_to_previous() {
            drop(chapter_list); // Release lock before seek
            self.seek(start_time)?;
            Ok(())
        } else {
            Err(EngineError::InvalidState("No previous chapter".to_string()))
        }
    }

    /// Jumps to a specific chapter by index
    pub fn go_to_chapter(&mut self, index: usize) -> EngineResult<()> {
        let mut chapter_list = self.chapter_list.lock().unwrap();

        if let Some(start_time) = chapter_list.go_to_chapter(index) {
            drop(chapter_list); // Release lock before seek
            self.seek(start_time)?;
            Ok(())
        } else {
            Err(EngineError::InvalidState(format!("Chapter {} not found", index)))
        }
    }

    /// Gets the current chapter info
    pub fn current_chapter(&self) -> Option<ChapterMarker> {
        let chapter_list = self.chapter_list.lock().unwrap();
        chapter_list.current_chapter().cloned()
    }

    /// Gets chapter progress string (e.g., "3/15")
    pub fn chapter_progress(&self) -> String {
        let chapter_list = self.chapter_list.lock().unwrap();
        chapter_list.chapter_progress()
    }

    /// Returns true if there are chapters loaded
    pub fn has_chapters(&self) -> bool {
        let chapter_list = self.chapter_list.lock().unwrap();
        chapter_list.has_chapters()
    }

    /// Gets all chapters
    pub fn chapters(&self) -> Vec<ChapterMarker> {
        let chapter_list = self.chapter_list.lock().unwrap();
        chapter_list.chapters().to_vec()
    }

    pub fn set_volume(&mut self, volume: f32) -> EngineResult<()> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(EngineError::InvalidSpeed(volume));
        }

        // Send volume command if playing
        if let Some(thread) = &self.playback_thread {
            thread.send_command(PlaybackCommand::SetVolume(volume))?;
        }

        // Update state
        let mut state = self.state.lock().unwrap();
        state.set_volume(volume);

        Ok(())
    }

    pub fn set_speed(&mut self, speed: PlaybackSpeed) -> EngineResult<()> {
        // Send speed command if playing
        if let Some(thread) = &self.playback_thread {
            thread.send_command(PlaybackCommand::SetSpeed(speed.value()))?;
        }

        // Update state
        let mut state = self.state.lock().unwrap();
        state.set_speed(speed);

        Ok(())
    }

    pub fn set_equalizer(&mut self, equalizer: Equalizer) -> EngineResult<()> {
        let mut state = self.state.lock().unwrap();
        state.set_equalizer(equalizer);
        Ok(())
    }

    pub fn status(&self) -> PlaybackStatus {
        let state = self.state.lock().unwrap();
        state.status()
    }

    pub fn position(&self) -> f64 {
        // Get position from playback thread if available
        let pos = if let Some(thread) = &self.playback_thread {
            thread.position()
        } else {
            let state = self.state.lock().unwrap();
            state.position()
        };

        // Update chapter list with current position
        if let Ok(mut chapter_list) = self.chapter_list.try_lock() {
            chapter_list.update_position(pos);
        }

        // Update state to keep it in sync
        if let Ok(mut state) = self.state.try_lock() {
            state.set_position(pos);
        }

        pos
    }

    pub fn volume(&self) -> f32 {
        let state = self.state.lock().unwrap();
        state.volume()
    }

    pub fn is_playing(&self) -> bool {
        if let Some(thread) = &self.playback_thread {
            thread.is_running() && self.status() == PlaybackStatus::Playing
        } else {
            false
        }
    }
}

impl Drop for MediaEngine {
    fn drop(&mut self) {
        // Ensure playback thread is stopped when engine is dropped
        if let Some(mut thread) = self.playback_thread.take() {
            thread.stop();
        }
    }
}

impl Default for MediaEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default MediaEngine")
    }
}

#[cfg(test)]
mod engine_tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = MediaEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_with_custom_config() {
        let engine = MediaEngine::with_config(());
        assert!(engine.is_ok());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let mut engine = MediaEngine::new().unwrap();
        let result = engine.load("nonexistent.mp3");
        assert!(result.is_err());
    }

    #[test]
    fn test_chapters_initially_empty() {
        let engine = MediaEngine::new().unwrap();
        assert!(!engine.has_chapters());
        assert_eq!(engine.chapter_progress(), "No chapters");
    }

    #[test]
    fn test_load_chapters() {
        let mut engine = MediaEngine::new().unwrap();

        let chapters = vec![
            ChapterMarker::new(0, "Chapter 1".to_string(), 0.0, 100.0),
            ChapterMarker::new(1, "Chapter 2".to_string(), 100.0, 200.0),
        ];

        engine.load_chapters(chapters).unwrap();
        assert!(engine.has_chapters());
        assert_eq!(engine.chapters().len(), 2);
    }

    #[test]
    fn test_next_chapter_without_chapters() {
        let mut engine = MediaEngine::new().unwrap();
        let result = engine.next_chapter();
        assert!(result.is_err());
    }

    #[test]
    fn test_previous_chapter_without_chapters() {
        let mut engine = MediaEngine::new().unwrap();
        let result = engine.previous_chapter();
        assert!(result.is_err());
    }

    #[test]
    fn test_chapter_progress_with_chapters() {
        let mut engine = MediaEngine::new().unwrap();

        let chapters = vec![
            ChapterMarker::new(0, "Ch1".to_string(), 0.0, 100.0),
            ChapterMarker::new(1, "Ch2".to_string(), 100.0, 200.0),
            ChapterMarker::new(2, "Ch3".to_string(), 200.0, 300.0),
        ];

        engine.load_chapters(chapters).unwrap();
        assert_eq!(engine.chapter_progress(), "1/3");
    }

    #[test]
    fn test_current_chapter() {
        let mut engine = MediaEngine::new().unwrap();

        let chapters = vec![
            ChapterMarker::new(0, "Introduction".to_string(), 0.0, 100.0),
            ChapterMarker::new(1, "Main Content".to_string(), 100.0, 200.0),
        ];

        engine.load_chapters(chapters).unwrap();

        let current = engine.current_chapter();
        assert!(current.is_some());
        assert_eq!(current.unwrap().title, "Introduction");
    }

    #[test]
    fn test_play_without_load() {
        let mut engine = MediaEngine::new().unwrap();
        let result = engine.play();
        assert!(result.is_err());
    }

    #[test]
    fn test_pause_without_load() {
        let mut engine = MediaEngine::new().unwrap();
        let result = engine.pause();
        assert!(result.is_err());
    }

    #[test]
    fn test_seek_without_load() {
        let mut engine = MediaEngine::new().unwrap();
        let result = engine.seek(10.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_volume() {
        let mut engine = MediaEngine::new().unwrap();
        assert!(engine.set_volume(0.5).is_ok());
        assert_eq!(engine.volume(), 0.5);
    }

    #[test]
    fn test_set_volume_invalid() {
        let mut engine = MediaEngine::new().unwrap();
        assert!(engine.set_volume(-0.1).is_err());
        assert!(engine.set_volume(1.1).is_err());
    }

    #[test]
    fn test_set_speed() {
        let mut engine = MediaEngine::new().unwrap();
        let speed = PlaybackSpeed::new(1.5).unwrap();
        assert!(engine.set_speed(speed).is_ok());
    }

    #[test]
    fn test_set_equalizer() {
        let mut engine = MediaEngine::new().unwrap();
        let equalizer = Equalizer::new_10_band();
        assert!(engine.set_equalizer(equalizer).is_ok());
    }

    #[test]
    fn test_status_accessor() {
        let engine = MediaEngine::new().unwrap();
        assert_eq!(engine.status(), PlaybackStatus::Stopped);
    }

    #[test]
    fn test_position_accessor() {
        let engine = MediaEngine::new().unwrap();
        assert_eq!(engine.position(), 0.0);
    }

    #[test]
    fn test_volume_accessor() {
        let engine = MediaEngine::new().unwrap();
        assert_eq!(engine.volume(), 1.0);
    }

    #[test]
    fn test_is_playing_without_load() {
        let engine = MediaEngine::new().unwrap();
        assert!(!engine.is_playing());
    }

    #[test]
    fn test_stop_resets_position() {
        let mut engine = MediaEngine::new().unwrap();
        engine.stop().unwrap();
        assert_eq!(engine.position(), 0.0);
        assert_eq!(engine.status(), PlaybackStatus::Stopped);
    }
}