//! Core media engine for audio playback

use crate::state::EngineState;
use crate::{AudioDecoder, AudioOutput, EngineError, EngineResult, Equalizer, PlaybackStatus};
use storystream_core::PlaybackSpeed;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct MediaEngine {
    state: Arc<Mutex<EngineState>>,
    decoder: Option<AudioDecoder>,
    output: Option<AudioOutput>,
}

impl MediaEngine {
    pub fn new() -> EngineResult<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(EngineState::default())),
            decoder: None,
            output: None,
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

        self.decoder = Some(AudioDecoder::new(path)?);
        self.output = Some(AudioOutput::new(44100, 2)?);

        let mut state = self.state.lock().unwrap();
        state.set_status(PlaybackStatus::Stopped);
        state.set_position(0.0);

        Ok(())
    }

    pub fn play(&mut self) -> EngineResult<()> {
        if self.decoder.is_none() {
            return Err(EngineError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file loaded",
            )));
        }

        let mut state = self.state.lock().unwrap();
        state.set_status(PlaybackStatus::Playing);
        Ok(())
    }

    pub fn pause(&mut self) -> EngineResult<()> {
        if self.decoder.is_none() {
            return Err(EngineError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file loaded",
            )));
        }

        let mut state = self.state.lock().unwrap();
        state.set_status(PlaybackStatus::Paused);
        Ok(())
    }

    pub fn stop(&mut self) -> EngineResult<()> {
        let mut state = self.state.lock().unwrap();
        state.set_status(PlaybackStatus::Stopped);
        state.set_position(0.0);
        Ok(())
    }

    pub fn seek(&mut self, position: f64) -> EngineResult<()> {
        if self.decoder.is_none() {
            return Err(EngineError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file loaded",
            )));
        }

        let mut state = self.state.lock().unwrap();
        state.set_position(position);
        Ok(())
    }

    pub fn set_volume(&mut self, volume: f32) -> EngineResult<()> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(EngineError::InvalidSpeed(volume));
        }

        let mut state = self.state.lock().unwrap();
        state.set_volume(volume);
        Ok(())
    }

    pub fn set_speed(&mut self, speed: PlaybackSpeed) -> EngineResult<()> {
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
        let state = self.state.lock().unwrap();
        state.position()
    }

    pub fn volume(&self) -> f32 {
        let state = self.state.lock().unwrap();
        state.volume()
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
}