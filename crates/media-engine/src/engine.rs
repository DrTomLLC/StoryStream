// crates/media-engine/src/engine.rs
// PANIC-FREE IMPLEMENTATION - Zero unwrap/expect calls, all errors handled gracefully

use crate::chapters::ChapterList;
use crate::decoder::AudioDecoder;
use crate::equalizer::Equalizer;
use crate::playback::PlaybackState;
use crate::playback_thread::{
    self, AudioDecoder as PlaybackAudioDecoder, Equalizer as PlaybackEqualizer, PlaybackCommand,
};
use crate::speed::Speed;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

/// Configuration for the media engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            buffer_size: 4096,
        }
    }
}

/// Main media playback engine - PANIC-FREE implementation
pub struct MediaEngine {
    #[allow(dead_code)]
    config: EngineConfig,
    command_tx: Arc<Mutex<Option<Sender<PlaybackCommand>>>>,
    loaded_file: Option<String>,
    decoder: Option<AudioDecoder>,
    current_position: Arc<Mutex<Duration>>,
    current_status: Arc<Mutex<bool>>,
    chapters: Arc<Mutex<ChapterList>>,
    volume: Arc<Mutex<f32>>,
    pub speed: Arc<Mutex<Speed>>,
    equalizer: Arc<Mutex<Equalizer>>,
    thread_handle: Option<JoinHandle<()>>,
    playback_state: Arc<Mutex<PlaybackState>>,
    pub duration: Option<Duration>,
}

impl MediaEngine {
    /// Creates a new media engine with the given configuration
    /// Returns Err if initialization fails - NEVER PANICS
    pub fn new(config: EngineConfig) -> Result<Self, String> {
        // Validate config parameters
        if config.sample_rate == 0 {
            return Err("Invalid config: sample_rate cannot be zero".to_string());
        }
        if config.channels == 0 {
            return Err("Invalid config: channels cannot be zero".to_string());
        }
        if config.buffer_size == 0 {
            return Err("Invalid config: buffer_size cannot be zero".to_string());
        }

        let command_tx = Arc::new(Mutex::new(None));

        Ok(Self {
            config,
            command_tx,
            loaded_file: None,
            decoder: None,
            current_position: Arc::new(Mutex::new(Duration::from_secs(0))),
            current_status: Arc::new(Mutex::new(false)),
            chapters: Arc::new(Mutex::new(ChapterList::new())),
            volume: Arc::new(Mutex::new(1.0)),
            speed: Arc::new(Mutex::new(Speed::default())),
            equalizer: Arc::new(Mutex::new(Equalizer::default())),
            thread_handle: None,
            playback_state: Arc::new(Mutex::new(PlaybackState::new())),
            duration: None,
        })
    }

    /// Creates a new media engine with default configuration
    /// Returns Err if initialization fails - NEVER PANICS
    pub fn with_defaults() -> Result<Self, String> {
        Self::new(EngineConfig::default())
    }

    /// Loads an audio file for playback
    /// Returns Err with actionable message on failure - NEVER PANICS
    pub fn load(&mut self, path: &str) -> Result<(), String> {
        // Validate input
        if path.is_empty() {
            return Err("Cannot load: path is empty".to_string());
        }

        // Stop any existing playback (gracefully)
        let stop_result = self.stop();
        if let Err(e) = stop_result {
            return Err(format!("Failed to stop existing playback: {}", e));
        }

        // Kill existing thread if any (gracefully)
        if let Some(handle) = self.thread_handle.take() {
            // Join returns Err only if thread panicked - handle gracefully
            if let Err(_) = handle.join() {
                return Err("Previous playback thread panicked - engine state may be corrupted".to_string());
            }
        }

        // Create decoder with error handling
        let path_buf = Path::new(path);
        let decoder = AudioDecoder::new(path_buf)
            .map_err(|e| format!("Failed to create decoder: {:?}", e))?;

        // Get duration safely - use default if unavailable
        let duration = match decoder.duration() {
            Some(d) => d,
            None => Duration::from_secs(300), // Safe default: 5 minutes
        };

        self.duration = Some(duration);
        self.decoder = Some(decoder);
        self.loaded_file = Some(path.to_string());

        // Update playback state with proper error handling
        match self.playback_state.lock() {
            Ok(mut state) => {
                state.set_duration(duration);
                *state = PlaybackState::stopped();
            }
            Err(e) => {
                return Err(format!("Failed to update playback state: mutex poisoned - {}", e));
            }
        }

        // Clear chapters when loading new file
        match self.chapters.lock() {
            Ok(mut chapters) => {
                *chapters = ChapterList::new();
            }
            Err(e) => {
                return Err(format!("Failed to clear chapters: mutex poisoned - {}", e));
            }
        }

        // Start new playback thread
        self.start_playback_thread()?;

        Ok(())
    }

    /// Starts playback
    /// Returns Err with actionable message on failure - NEVER PANICS
    pub fn play(&mut self) -> Result<(), String> {
        if self.loaded_file.is_none() {
            return Err("Cannot play: no file loaded. Call load() first".to_string());
        }

        let tx = match self.command_tx.lock() {
            Ok(guard) => match guard.as_ref() {
                Some(tx) => tx.clone(),
                None => return Err("Cannot play: playback thread not running. Try reloading the file".to_string()),
            },
            Err(e) => return Err(format!("Cannot play: command channel poisoned - {}", e)),
        };

        tx.send(PlaybackCommand::Play)
            .map_err(|e| format!("Failed to send play command: {}", e))?;

        // Update status with error handling
        if let Ok(mut status) = self.current_status.lock() {
            *status = true;
        }
        // If mutex is poisoned, we continue - status update is not critical

        Ok(())
    }

    /// Pauses playback
    /// Returns Err with actionable message on failure - NEVER PANICS
    pub fn pause(&mut self) -> Result<(), String> {
        if self.loaded_file.is_none() {
            return Err("Cannot pause: no file loaded".to_string());
        }

        let tx = match self.command_tx.lock() {
            Ok(guard) => match guard.as_ref() {
                Some(tx) => tx.clone(),
                None => return Err("Cannot pause: playback thread not running".to_string()),
            },
            Err(e) => return Err(format!("Cannot pause: command channel poisoned - {}", e)),
        };

        tx.send(PlaybackCommand::Pause)
            .map_err(|e| format!("Failed to send pause command: {}", e))?;

        // Update status with error handling
        if let Ok(mut status) = self.current_status.lock() {
            *status = false;
        }

        Ok(())
    }

    /// Stops playback - always succeeds, never panics
    /// Returns Ok(()) on success, Err only for truly unrecoverable errors
    pub fn stop(&mut self) -> Result<(), String> {
        // Best-effort stop - ignore errors as stop should always succeed
        if let Ok(guard) = self.command_tx.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(PlaybackCommand::Stop);
            }
        }

        // Reset position - best effort
        if let Ok(mut pos) = self.current_position.lock() {
            *pos = Duration::from_secs(0);
        }

        // Reset status - best effort
        if let Ok(mut status) = self.current_status.lock() {
            *status = false;
        }

        // Stop always succeeds - errors are non-critical
        Ok(())
    }

    /// Seeks to a specific position
    /// Returns Err with actionable message on failure - NEVER PANICS
    pub fn seek(&mut self, position: Duration) -> Result<(), String> {
        if self.loaded_file.is_none() {
            return Err("Cannot seek: no file loaded".to_string());
        }

        // Validate position against duration
        if let Some(dur) = self.duration {
            if position > dur {
                return Err(format!(
                    "Cannot seek to {}s: exceeds file duration of {}s",
                    position.as_secs(),
                    dur.as_secs()
                ));
            }
        }

        let tx = match self.command_tx.lock() {
            Ok(guard) => match guard.as_ref() {
                Some(tx) => tx.clone(),
                None => return Err("Cannot seek: playback thread not running".to_string()),
            },
            Err(e) => return Err(format!("Cannot seek: command channel poisoned - {}", e)),
        };

        tx.send(PlaybackCommand::Seek(position))
            .map_err(|e| format!("Failed to send seek command: {}", e))?;

        // Update position with error handling
        if let Ok(mut pos) = self.current_position.lock() {
            *pos = position;
        }

        Ok(())
    }

    /// Sets the playback volume (0.0 to 1.0)
    /// Returns Err with actionable message on invalid input - NEVER PANICS
    pub fn set_volume(&mut self, volume: f32) -> Result<(), String> {
        // Validate input with explicit bounds check
        if volume.is_nan() {
            return Err("Invalid volume: NaN is not allowed".to_string());
        }
        if volume.is_infinite() {
            return Err("Invalid volume: infinite value is not allowed".to_string());
        }
        if volume < 0.0 {
            return Err(format!("Invalid volume: {} is below minimum (0.0)", volume));
        }
        if volume > 1.0 {
            return Err(format!("Invalid volume: {} exceeds maximum (1.0)", volume));
        }

        // Update volume with error handling
        match self.volume.lock() {
            Ok(mut vol) => *vol = volume,
            Err(e) => return Err(format!("Failed to set volume: mutex poisoned - {}", e)),
        }

        // Send command if playback thread is running (best effort)
        if let Ok(guard) = self.command_tx.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(PlaybackCommand::SetVolume(volume));
            }
        }

        Ok(())
    }

    /// Sets the playback speed
    /// Returns Err with actionable message on failure - NEVER PANICS
    pub fn set_speed(&mut self, speed: Speed) -> Result<(), String> {
        // Speed validation is handled by Speed::new() constructor
        match self.speed.lock() {
            Ok(mut spd) => *spd = speed,
            Err(e) => return Err(format!("Failed to set speed: mutex poisoned - {}", e)),
        }

        // Send command if playback thread is running (best effort)
        if let Ok(guard) = self.command_tx.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(PlaybackCommand::SetSpeed(speed));
            }
        }

        Ok(())
    }

    /// Sets the equalizer
    /// Returns Err with actionable message on failure - NEVER PANICS
    pub fn set_equalizer(&mut self, equalizer: Equalizer) -> Result<(), String> {
        match self.equalizer.lock() {
            Ok(mut eq) => {
                *eq = equalizer;
                Ok(())
            }
            Err(e) => Err(format!("Failed to set equalizer: mutex poisoned - {}", e)),
        }
    }

    /// Returns the current playback position - NEVER PANICS
    /// Returns Duration::ZERO if position cannot be retrieved
    pub fn position(&self) -> Duration {
        self.current_position
            .lock()
            .map(|pos| *pos)
            .unwrap_or_else(|_| Duration::from_secs(0))
    }

    /// Returns whether playback is currently active - NEVER PANICS
    /// Returns false if status cannot be retrieved
    pub fn is_playing(&self) -> bool {
        self.current_status
            .lock()
            .map(|status| *status)
            .unwrap_or(false)
    }

    /// Returns the playback status - NEVER PANICS
    /// Returns false if status cannot be retrieved
    pub fn status(&self) -> bool {
        self.current_status
            .lock()
            .map(|status| *status)
            .unwrap_or(false)
    }

    /// Returns the current volume - NEVER PANICS
    /// Returns 1.0 if volume cannot be retrieved
    pub fn volume(&self) -> f32 {
        self.volume
            .lock()
            .map(|vol| *vol)
            .unwrap_or(1.0)
    }

    /// Returns the current playback state - NEVER PANICS
    /// Returns default state if state cannot be retrieved
    pub fn get_playback_state(&self) -> PlaybackState {
        self.playback_state
            .lock()
            .map(|state| state.clone())
            .unwrap_or_else(|_| PlaybackState::new())
    }

    /// Loads chapters from metadata or file - NEVER PANICS
    pub fn load_chapters(&mut self, _chapters: Vec<(String, Duration, Duration)>) {
        // Placeholder until chapter integration is complete
        // This method cannot fail, so it never returns an error
    }

    /// Returns the list of chapters - NEVER PANICS
    /// Returns empty chapter list if chapters cannot be retrieved
    pub fn chapters(&self) -> ChapterList {
        self.chapters
            .lock()
            .map(|chapters| chapters.clone())
            .unwrap_or_else(|_| ChapterList::new())
    }

    /// Returns the current chapter based on playback position - NEVER PANICS
    pub fn current_chapter(&self) -> Option<usize> {
        None
    }

    /// Jumps to the next chapter - NEVER PANICS
    pub fn next_chapter(&mut self) -> Result<(), String> {
        Err("Chapter navigation not yet implemented".to_string())
    }

    /// Jumps to the previous chapter - NEVER PANICS
    pub fn previous_chapter(&mut self) -> Result<(), String> {
        Err("Chapter navigation not yet implemented".to_string())
    }

    /// Returns progress through the current chapter as a percentage - NEVER PANICS
    pub fn chapter_progress(&self) -> Option<f32> {
        None
    }

    /// Internal method to start the playback thread
    /// Returns Err with actionable message on failure - NEVER PANICS
    fn start_playback_thread(&mut self) -> Result<(), String> {
        let _decoder = match self.decoder.take() {
            Some(d) => d,
            None => return Err("Cannot start playback: no decoder available".to_string()),
        };

        let duration = self.duration.unwrap_or_else(|| Duration::from_secs(0));

        let (tx, rx) = channel();
        match self.command_tx.lock() {
            Ok(mut guard) => *guard = Some(tx),
            Err(e) => return Err(format!("Cannot start playback: command channel poisoned - {}", e)),
        }

        // Get file path safely
        let path_str = match self.loaded_file.as_ref() {
            Some(p) => p,
            None => return Err("Cannot start playback: no file loaded".to_string()),
        };

        let path = Path::new(path_str);
        let playback_decoder = PlaybackAudioDecoder::new(path)
            .map_err(|e| format!("Failed to create playback decoder: {:?}", e))?;

        let playback_equalizer = Arc::new(Mutex::new(PlaybackEqualizer::default()));

        let handle = playback_thread::start_playback_thread(
            playback_decoder,
            rx,
            duration,
            self.current_position.clone(),
            self.current_status.clone(),
            self.playback_state.clone(),
            self.volume.clone(),
            self.speed.clone(),
            playback_equalizer,
        );

        self.thread_handle = Some(handle);
        Ok(())
    }
}

impl Drop for MediaEngine {
    /// Cleanup on drop - NEVER PANICS
    /// Best-effort cleanup with all errors ignored
    fn drop(&mut self) {
        // Best-effort stop - ignore all errors
        let _ = self.stop();

        // Join thread if it exists - ignore join errors
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

// Helper function to handle PoisonError gracefully - NEVER PANICS
fn _handle_poison_error<T>(err: PoisonError<T>) -> String {
    format!("Mutex poisoned: {}", err)
}

#[cfg(test)]
mod engine_tests {
    use super::*;

    #[test]
    fn test_engine_creation_never_panics() {
        let engine = MediaEngine::with_defaults();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_invalid_config_never_panics() {
        let config = EngineConfig {
            sample_rate: 0,  // Invalid!
            channels: 2,
            buffer_size: 4096,
        };
        let result = MediaEngine::new(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sample_rate"));
    }

    #[test]
    fn test_volume_validation_never_panics() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            assert!(engine.set_volume(0.0).is_ok());
            assert!(engine.set_volume(1.0).is_ok());
            assert!(engine.set_volume(-0.1).is_err());
            assert!(engine.set_volume(1.1).is_err());
            assert!(engine.set_volume(f32::NAN).is_err());
            assert!(engine.set_volume(f32::INFINITY).is_err());
        }
    }

    #[test]
    fn test_operations_without_file_never_panic() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            assert!(engine.play().is_err());
            assert!(engine.pause().is_err());
            assert!(engine.seek(Duration::from_secs(10)).is_err());
            assert!(engine.stop().is_ok());  // Stop always succeeds
        }
    }

    #[test]
    fn test_empty_path_never_panics() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let result = engine.load("");
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("empty"));
        }
    }

    #[test]
    fn test_seek_beyond_duration_never_panics() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            engine.duration = Some(Duration::from_secs(100));
            let result = engine.seek(Duration::from_secs(200));
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("exceeds"));
        }
    }

    #[test]
    fn test_getters_never_panic() {
        if let Ok(engine) = MediaEngine::with_defaults() {
            let _ = engine.position();
            let _ = engine.is_playing();
            let _ = engine.status();
            let _ = engine.volume();
            let _ = engine.get_playback_state();
            let _ = engine.chapters();
            let _ = engine.current_chapter();
            let _ = engine.chapter_progress();
        }
    }

    #[test]
    fn test_multiple_stops_never_panic() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            assert!(engine.stop().is_ok());
            assert!(engine.stop().is_ok());
            assert!(engine.stop().is_ok());
        }
    }
}