// FILE: src/engine.rs
// ============================================================================

use crate::{EngineError, EngineResult, EngineState, Equalizer, PlaybackSpeed, PlaybackStatus};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Main audio engine
pub struct AudioEngine {
    state: Arc<Mutex<EngineState>>,
    config: EngineConfig,
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Default volume (0-100)
    pub default_volume: u8,
    /// Default playback speed
    pub default_speed: PlaybackSpeed,
    /// Enable equalizer by default
    pub enable_equalizer: bool,
    /// Buffer size in samples
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
    /// Creates a new audio engine with default configuration
    pub fn new() -> Self {
        Self::with_config(EngineConfig::default())
    }

    /// Creates a new audio engine with custom configuration
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
        }
    }

    /// Loads an audio file
    pub fn load(&self, path: &Path) -> EngineResult<()> {
        // Validate file exists and format is supported
        if !path.exists() {
            return Err(EngineError::DecodeError("File not found".to_string()));
        }

        // Format detection would happen here
        let format = storystream_media_formats::AudioFormat::from_path(path)
            .ok_or_else(|| EngineError::UnsupportedFormat(path.display().to_string()))?;

        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        state.current_file = Some(path.to_path_buf());
        state.position = Duration::ZERO;
        // In real implementation, would decode file to get actual duration
        state.duration = Some(Duration::from_secs(3600)); // Placeholder
        state.status = PlaybackStatus::Stopped;

        Ok(())
    }

    /// Starts playback
    pub fn play(&self) -> EngineResult<()> {
        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        if state.current_file.is_none() {
            return Err(EngineError::NoAudioLoaded);
        }

        state.status = PlaybackStatus::Playing;
        Ok(())
    }

    /// Pauses playback
    pub fn pause(&self) -> EngineResult<()> {
        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        if state.current_file.is_none() {
            return Err(EngineError::NoAudioLoaded);
        }

        state.status = PlaybackStatus::Paused;
        Ok(())
    }

    /// Stops playback and resets position
    pub fn stop(&self) -> EngineResult<()> {
        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        state.status = PlaybackStatus::Stopped;
        state.position = Duration::ZERO;
        Ok(())
    }

    /// Seeks to a specific position
    pub fn seek(&self, position: Duration) -> EngineResult<()> {
        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        if state.current_file.is_none() {
            return Err(EngineError::NoAudioLoaded);
        }

        if let Some(duration) = state.duration {
            if position > duration {
                return Err(EngineError::InvalidPosition(position));
            }
        }

        state.position = position;
        Ok(())
    }

    /// Sets the volume (0-100)
    pub fn set_volume(&self, volume: u8) -> EngineResult<()> {
        if volume > 100 {
            return Err(EngineError::InvalidVolume(volume));
        }

        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        state.volume = volume;
        Ok(())
    }

    /// Sets the playback speed
    pub fn set_speed(&self, speed: PlaybackSpeed) -> EngineResult<()> {
        let mut state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        state.speed = speed;
        Ok(())
    }

    /// Gets the current playback status
    pub fn status(&self) -> EngineResult<PlaybackStatus> {
        let state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        Ok(state.status)
    }

    /// Gets the current position
    pub fn position(&self) -> EngineResult<Duration> {
        let state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        Ok(state.position)
    }

    /// Gets the total duration
    pub fn duration(&self) -> EngineResult<Option<Duration>> {
        let state = self.state.lock().map_err(|_| {
            EngineError::InvalidState("Failed to acquire state lock".to_string())
        })?;

        Ok(state.duration)
    }

    /// Gets the current volume
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

    #[test]
    fn test_engine_creation() {
        let engine = AudioEngine::new();
        assert_eq!(engine.status().unwrap(), PlaybackStatus::Stopped);
    }

    #[test]
    fn test_engine_with_config() {
        let config = EngineConfig {
            default_volume: 85,
            ..Default::default()
        };
        let engine = AudioEngine::with_config(config);
        assert_eq!(engine.volume().unwrap(), 85);
    }

    #[test]
    fn test_play_without_audio() {
        let engine = AudioEngine::new();
        let result = engine.play();
        assert_eq!(result, Err(EngineError::NoAudioLoaded));
    }

    #[test]
    fn test_pause_without_audio() {
        let engine = AudioEngine::new();
        let result = engine.pause();
        assert_eq!(result, Err(EngineError::NoAudioLoaded));
    }

    #[test]
    fn test_seek_without_audio() {
        let engine = AudioEngine::new();
        let result = engine.seek(Duration::from_secs(10));
        assert_eq!(result, Err(EngineError::NoAudioLoaded));
    }

    #[test]
    fn test_volume_control() {
        let engine = AudioEngine::new();
        assert!(engine.set_volume(50).is_ok());
        assert_eq!(engine.volume().unwrap(), 50);

        assert_eq!(
            engine.set_volume(101),
            Err(EngineError::InvalidVolume(101))
        );
    }

    #[test]
    fn test_speed_control() {
        let engine = AudioEngine::new();
        let speed = PlaybackSpeed::new(1.5).unwrap();
        assert!(engine.set_speed(speed).is_ok());
    }

    #[test]
    fn test_stop_resets_position() {
        let engine = AudioEngine::new();
        assert!(engine.stop().is_ok());
        assert_eq!(engine.position().unwrap(), Duration::ZERO);
    }

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert_eq!(config.default_volume, 70);
        assert_eq!(config.buffer_size, 4096);
    }
}
