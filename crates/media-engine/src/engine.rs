// FILE: crates/media-engine/src/engine.rs

use crate::{EngineError, EngineResult, EngineState, Equalizer, PlaybackSpeed, PlaybackStatus};
use crate::playback_thread::{PlaybackThread, PlaybackCommand};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AudioEngine {
    state: Arc<Mutex<EngineState>>,
    #[allow(dead_code)]
    config: EngineConfig,
    playback_thread: Mutex<Option<PlaybackThread>>,
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub default_volume: u8,
    pub default_speed: PlaybackSpeed,
    pub enable_equalizer: bool,
    pub buffer_size: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            default_volume: 70,
            default_speed: PlaybackSpeed::default(),
            enable_equalizer: false,
            buffer_size: 4096,
        }
    }
}

impl AudioEngine {
    pub fn new() -> Self {
        Self::with_config(EngineConfig::default())
    }

    pub fn with_config(config: EngineConfig) -> Self {
        let state = EngineState {
            status: PlaybackStatus::Stopped,
            current_file: None,
            position: Duration::ZERO,
            duration: None,
            volume: config.default_volume,
            speed: config.default_speed,
            equalizer: Equalizer::default(),
            chapters: Vec::new(),
            current_chapter: None,
        };

        Self {
            state: Arc::new(Mutex::new(state)),
            config,
            playback_thread: Mutex::new(None),
        }
    }

    pub fn load(&self, path: &Path) -> EngineResult<()> {
        if !path.exists() {
            return Err(EngineError::DecodeError("File not found".to_string()));
        }

        let _format = storystream_media_formats::AudioFormat::from_path(path)
            .ok_or_else(|| EngineError::UnsupportedFormat(path.display().to_string()))?;

        // Stop any existing playback
        if let Ok(mut thread_guard) = self.playback_thread.lock() {
            *thread_guard = None;
        }

        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        state.current_file = Some(path.to_path_buf());
        state.position = Duration::ZERO;
        state.duration = Some(Duration::from_secs(3600));
        state.status = PlaybackStatus::Stopped;

        Ok(())
    }

    pub fn play(&self) -> EngineResult<()> {
        let state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        let path = state
            .current_file
            .as_ref()
            .ok_or_else(|| EngineError::InvalidState("No file loaded".to_string()))?
            .clone();

        drop(state);

        // Start playback thread if not already running
        let mut thread_guard = self.playback_thread.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire thread lock".to_string())
        })?;

        if thread_guard.is_none() {
            let thread = PlaybackThread::start(&path)?;
            *thread_guard = Some(thread);
        }

        // Send play command
        if let Some(thread) = thread_guard.as_ref() {
            thread.send_command(PlaybackCommand::Play)?;
        }

        drop(thread_guard);

        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;
        state.status = PlaybackStatus::Playing;

        Ok(())
    }

    pub fn pause(&self) -> EngineResult<()> {
        let thread_guard = self.playback_thread.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire thread lock".to_string())
        })?;

        if let Some(thread) = thread_guard.as_ref() {
            thread.send_command(PlaybackCommand::Pause)?;
        }

        drop(thread_guard);

        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;
        state.status = PlaybackStatus::Paused;

        Ok(())
    }

    pub fn stop(&self) -> EngineResult<()> {
        let mut thread_guard = self.playback_thread.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire thread lock".to_string())
        })?;

        *thread_guard = None;

        drop(thread_guard);

        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;
        state.status = PlaybackStatus::Stopped;
        state.position = Duration::ZERO;

        Ok(())
    }

    pub fn seek(&self, position: Duration) -> EngineResult<()> {
        let state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        if state.current_file.is_none() {
            return Err(EngineError::InvalidState("No file loaded".to_string()));
        }

        drop(state);

        let thread_guard = self.playback_thread.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire thread lock".to_string())
        })?;

        if let Some(thread) = thread_guard.as_ref() {
            thread.send_command(PlaybackCommand::Seek(position.as_secs_f64()))?;
        }

        drop(thread_guard);

        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;
        state.position = position;

        Ok(())
    }

    pub fn set_volume(&self, volume: u8) -> EngineResult<()> {
        if volume > 100 {
            return Err(EngineError::InvalidState("Volume must be 0-100".to_string()));
        }

        let thread_guard = self.playback_thread.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire thread lock".to_string())
        })?;

        if let Some(thread) = thread_guard.as_ref() {
            thread.send_command(PlaybackCommand::SetVolume(volume as f32 / 100.0))?;
        }

        drop(thread_guard);

        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;
        state.volume = volume;

        Ok(())
    }

    pub fn set_speed(&self, speed: PlaybackSpeed) -> EngineResult<()> {
        let thread_guard = self.playback_thread.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire thread lock".to_string())
        })?;

        if let Some(thread) = thread_guard.as_ref() {
            thread.send_command(PlaybackCommand::SetSpeed(speed.value()))?;
        }

        drop(thread_guard);

        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;
        state.speed = speed;

        Ok(())
    }

    pub fn set_equalizer(&self, equalizer: Equalizer) -> EngineResult<()> {
        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;
        state.equalizer = equalizer;
        Ok(())
    }

    pub fn status(&self) -> EngineResult<PlaybackStatus> {
        let state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;
        Ok(state.status)
    }

    pub fn position(&self) -> EngineResult<Duration> {
        let thread_guard = self.playback_thread.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire thread lock".to_string())
        })?;

        if let Some(thread) = thread_guard.as_ref() {
            if thread.is_running() {
                let pos_secs = thread.position();
                return Ok(Duration::from_secs_f64(pos_secs));
            }
        }

        drop(thread_guard);

        let state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;
        Ok(state.position)
    }

    pub fn volume(&self) -> EngineResult<u8> {
        let state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;
        Ok(state.volume)
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod engine_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_engine_creation() {
        let engine = AudioEngine::new();
        assert!(engine.status().is_ok());
    }

    #[test]
    fn test_engine_with_custom_config() {
        let config = EngineConfig {
            default_volume: 50,
            default_speed: PlaybackSpeed::new(1.5).expect("Valid speed"),
            enable_equalizer: true,
            buffer_size: 8192,
        };
        let engine = AudioEngine::with_config(config);
        assert_eq!(engine.volume().expect("Should get volume"), 50);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let engine = AudioEngine::new();
        let path = PathBuf::from("/nonexistent/file.mp3");
        let result = engine.load(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_play_without_load() {
        let engine = AudioEngine::new();
        let result = engine.play();
        assert!(result.is_err());
    }

    #[test]
    fn test_pause_without_load() {
        let engine = AudioEngine::new();
        let result = engine.pause();
        assert!(result.is_ok());
    }

    #[test]
    fn test_seek_without_load() {
        let engine = AudioEngine::new();
        let result = engine.seek(Duration::from_secs(10));
        assert!(result.is_err());
    }

    #[test]
    fn test_set_volume() {
        let engine = AudioEngine::new();
        let result = engine.set_volume(80);
        assert!(result.is_ok());
        assert_eq!(engine.volume().expect("Should get volume"), 80);
    }

    #[test]
    fn test_set_volume_invalid() {
        let engine = AudioEngine::new();
        let result = engine.set_volume(150);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_speed() {
        let engine = AudioEngine::new();
        let speed = PlaybackSpeed::new(1.25).expect("Valid speed");
        let result = engine.set_speed(speed);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_equalizer() {
        let engine = AudioEngine::new();
        let eq = Equalizer::default();
        let result = engine.set_equalizer(eq);
        assert!(result.is_ok());
    }

    #[test]
    fn test_status_accessor() {
        let engine = AudioEngine::new();
        let status = engine.status().expect("Should get status");
        assert_eq!(status, PlaybackStatus::Stopped);
    }

    #[test]
    fn test_position_accessor() {
        let engine = AudioEngine::new();
        let pos = engine.position().expect("Should get position");
        assert_eq!(pos, Duration::ZERO);
    }

    #[test]
    fn test_volume_accessor() {
        let engine = AudioEngine::new();
        let vol = engine.volume().expect("Should get volume");
        assert_eq!(vol, 70);
    }
}