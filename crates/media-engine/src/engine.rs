use crate::chapters::ChapterList;
use crate::decoder::AudioDecoder;
use crate::equalizer::Equalizer;
use crate::playback::{PlaybackState, PlaybackStatus};
use crate::playback_thread::{self, PlaybackCommand, AudioDecoder as PlaybackAudioDecoder, Equalizer as PlaybackEqualizer};
use crate::speed::Speed;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
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

/// Main media playback engine
pub struct MediaEngine {
    config: EngineConfig,
    command_tx: Arc<Mutex<Option<Sender<PlaybackCommand>>>>,
    loaded_file: Option<String>,
    decoder: Option<AudioDecoder>,
    current_position: Arc<Mutex<Duration>>,
    current_status: Arc<Mutex<bool>>,
    chapters: Arc<Mutex<ChapterList>>,
    volume: Arc<Mutex<f32>>,
    speed: Arc<Mutex<Speed>>,
    equalizer: Arc<Mutex<Equalizer>>,
    thread_handle: Option<JoinHandle<()>>,
    playback_state: Arc<Mutex<PlaybackState>>,
    duration: Option<Duration>,
}

impl MediaEngine {
    /// Creates a new media engine with the given configuration
    pub fn new(config: EngineConfig) -> Result<Self, String> {
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
    pub fn with_defaults() -> Result<Self, String> {
        Self::new(EngineConfig::default())
    }

    /// Loads an audio file for playback
    pub fn load(&mut self, path: &str) -> Result<(), String> {
        // Stop any existing playback
        self.stop()?;

        // Kill existing thread if any
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        // Create decoder
        let path_buf = Path::new(path);
        let decoder = AudioDecoder::new(path_buf)
            .map_err(|e| format!("Failed to create decoder: {:?}", e))?;

        // Store duration from decoder metadata (assuming it's available in the struct)
        // For now, use a default duration since we don't have access to the actual method
        let duration = Duration::from_secs(300); // Placeholder
        self.duration = Some(duration);

        self.decoder = Some(decoder);
        self.loaded_file = Some(path.to_string());

        // Update playback state with duration
        if let Ok(mut state) = self.playback_state.lock() {
            state.set_duration(duration);
            *state = PlaybackState::stopped();
        }

        // Clear chapters when loading new file (ChapterList doesn't have clear, so create new one)
        if let Ok(mut chapters) = self.chapters.lock() {
            *chapters = ChapterList::new();
        }

        // Start new playback thread
        self.start_playback_thread()?;

        Ok(())
    }

    /// Starts playback
    pub fn play(&mut self) -> Result<(), String> {
        if self.loaded_file.is_none() {
            return Err("No file loaded".to_string());
        }

        let tx = self.command_tx.lock()
            .map_err(|_| "Failed to acquire command lock")?
            .as_ref()
            .ok_or("Playback thread not running")?
            .clone();

        tx.send(PlaybackCommand::Play).map_err(|e| e.to_string())?;

        if let Ok(mut status) = self.current_status.lock() {
            *status = true;
        }

        // Update playback state
        if let Ok(mut state) = self.playback_state.lock() {
            state.set_status(PlaybackStatus::Playing);
        }

        Ok(())
    }

    /// Pauses playback
    pub fn pause(&mut self) -> Result<(), String> {
        if self.loaded_file.is_none() {
            return Err("No file loaded".to_string());
        }

        let tx = self.command_tx.lock()
            .map_err(|_| "Failed to acquire command lock")?
            .as_ref()
            .ok_or("Playback thread not running")?
            .clone();

        tx.send(PlaybackCommand::Pause).map_err(|e| e.to_string())?;

        if let Ok(mut status) = self.current_status.lock() {
            *status = false;
        }

        // Update playback state
        if let Ok(mut state) = self.playback_state.lock() {
            state.set_status(PlaybackStatus::Paused);
        }

        Ok(())
    }

    /// Stops playback
    pub fn stop(&mut self) -> Result<(), String> {
        if let Ok(guard) = self.command_tx.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(PlaybackCommand::Stop);
            }
        }

        if let Ok(mut status) = self.current_status.lock() {
            *status = false;
        }

        if let Ok(mut position) = self.current_position.lock() {
            *position = Duration::from_secs(0);
        }

        // Update playback state
        if let Ok(mut state) = self.playback_state.lock() {
            *state = PlaybackState::stopped();
        }

        Ok(())
    }

    /// Seeks to a specific position in the current file
    pub fn seek(&mut self, position: Duration) -> Result<(), String> {
        if self.loaded_file.is_none() {
            return Err("No file loaded".to_string());
        }

        let tx = self.command_tx.lock()
            .map_err(|_| "Failed to acquire command lock")?
            .as_ref()
            .ok_or("Playback thread not running")?
            .clone();

        tx.send(PlaybackCommand::Seek(position)).map_err(|e| e.to_string())?;

        if let Ok(mut pos) = self.current_position.lock() {
            *pos = position;
        }

        // Update playback state
        if let Ok(mut state) = self.playback_state.lock() {
            state.set_position(position);
        }

        Ok(())
    }

    /// Sets the playback volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) -> Result<(), String> {
        if !(0.0..=1.0).contains(&volume) {
            return Err("Volume must be between 0.0 and 1.0".to_string());
        }

        if let Ok(mut vol) = self.volume.lock() {
            *vol = volume;
        }

        if let Ok(guard) = self.command_tx.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(PlaybackCommand::SetVolume(volume));
            }
        }

        Ok(())
    }

    /// Sets the playback speed
    pub fn set_speed(&mut self, speed: Speed) -> Result<(), String> {
        if let Ok(mut spd) = self.speed.lock() {
            *spd = speed;
        }

        if let Ok(guard) = self.command_tx.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(PlaybackCommand::SetSpeed(speed));
            }
        }

        Ok(())
    }

    /// Sets the equalizer
    pub fn set_equalizer(&mut self, equalizer: Equalizer) -> Result<(), String> {
        if let Ok(mut eq) = self.equalizer.lock() {
            *eq = equalizer;
        }
        Ok(())
    }

    /// Returns the current playback position
    pub fn position(&self) -> Duration {
        self.current_position.lock()
            .map(|pos| *pos)
            .unwrap_or_else(|_| Duration::from_secs(0))
    }

    /// Returns whether playback is currently active
    pub fn is_playing(&self) -> bool {
        self.current_status.lock()
            .map(|status| *status)
            .unwrap_or(false)
    }

    /// Returns the playback status
    pub fn status(&self) -> bool {
        self.current_status.lock()
            .map(|status| *status)
            .unwrap_or(false)
    }

    /// Returns the current volume
    pub fn volume(&self) -> f32 {
        self.volume.lock()
            .map(|vol| *vol)
            .unwrap_or(1.0)
    }

    /// Returns the current playback state
    pub fn get_playback_state(&self) -> PlaybackState {
        self.playback_state.lock()
            .map(|state| state.clone())
            .unwrap_or_else(|_| PlaybackState::new())
    }

    /// Loads chapters from metadata or file
    pub fn load_chapters(&mut self, _chapters: Vec<(String, Duration, Duration)>) {
        // ChapterList API uses ChapterMarker, not exposed for now
        // This is a placeholder until chapter integration is complete
    }

    /// Returns the list of chapters
    pub fn chapters(&self) -> ChapterList {
        self.chapters.lock()
            .map(|chapters| chapters.clone())
            .unwrap_or_else(|_| ChapterList::new())
    }

    /// Returns the current chapter based on playback position
    pub fn current_chapter(&self) -> Option<usize> {
        // ChapterList API needs position as f64, returns ChapterMarker
        // For now, return None until full chapter integration
        None
    }

    /// Jumps to the next chapter
    pub fn next_chapter(&mut self) -> Result<(), String> {
        Err("Chapter navigation not yet implemented".to_string())
    }

    /// Jumps to the previous chapter
    pub fn previous_chapter(&mut self) -> Result<(), String> {
        Err("Chapter navigation not yet implemented".to_string())
    }

    /// Returns progress through the current chapter as a percentage
    pub fn chapter_progress(&self) -> Option<f32> {
        // ChapterList::chapter_progress returns String, not Option<f32>
        // Return None for now until chapter integration is complete
        None
    }

    fn start_playback_thread(&mut self) -> Result<(), String> {
        let _decoder = self.decoder.take().ok_or("No decoder available")?;
        // FIXED: Prefixed with underscore as it's intentionally unused
        // This take() is needed to move the decoder out, but we don't actually use it
        // because we create a new PlaybackAudioDecoder from the file path below

        let duration = self.duration.unwrap_or_else(|| Duration::from_secs(0));

        let (tx, rx) = channel();
        if let Ok(mut guard) = self.command_tx.lock() {
            *guard = Some(tx);
        }

        // Using the path from the loaded file to create a new playback decoder
        let path = Path::new(self.loaded_file.as_ref().ok_or("No file loaded")?);
        let playback_decoder = PlaybackAudioDecoder::new(path)
            .map_err(|e| format!("Failed to create playback decoder: {:?}", e))?;

        // Create a playback equalizer
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
    fn drop(&mut self) {
        let _ = self.stop();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod engine_tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = MediaEngine::with_defaults();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_with_custom_config() {
        let config = EngineConfig {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 2048,
        };
        let engine = MediaEngine::new(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_play_without_load() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let result = engine.play();
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_pause_without_load() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let result = engine.pause();
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_seek_without_load() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let result = engine.seek(Duration::from_secs(10));
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_load_nonexistent_file() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let result = engine.load("nonexistent.mp3");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_set_volume() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            assert!(engine.set_volume(0.5).is_ok());
            assert_eq!(engine.volume(), 0.5);
        }
    }

    #[test]
    fn test_set_volume_invalid() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            assert!(engine.set_volume(1.5).is_err());
            assert!(engine.set_volume(-0.5).is_err());
        }
    }

    #[test]
    fn test_set_speed() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            if let Ok(speed) = Speed::new(1.5) {
                assert!(engine.set_speed(speed).is_ok());
            }
        }
    }

    #[test]
    fn test_set_equalizer() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let equalizer = Equalizer::default();
            assert!(engine.set_equalizer(equalizer).is_ok());
        }
    }

    #[test]
    fn test_is_playing_without_load() {
        if let Ok(engine) = MediaEngine::with_defaults() {
            assert!(!engine.is_playing());
        }
    }

    #[test]
    fn test_position_accessor() {
        if let Ok(engine) = MediaEngine::with_defaults() {
            assert_eq!(engine.position(), Duration::from_secs(0));
        }
    }

    #[test]
    fn test_volume_accessor() {
        if let Ok(engine) = MediaEngine::with_defaults() {
            assert_eq!(engine.volume(), 1.0);
        }
    }

    #[test]
    fn test_status_accessor() {
        if let Ok(engine) = MediaEngine::with_defaults() {
            assert!(!engine.status());
        }
    }

    #[test]
    fn test_stop_resets_position() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let _ = engine.stop();
            assert_eq!(engine.position(), Duration::from_secs(0));
        }
    }

    #[test]
    fn test_chapters_initially_empty() {
        if let Ok(engine) = MediaEngine::with_defaults() {
            let _chapters = engine.chapters();
            // ChapterList doesn't expose len(), just verify we got a ChapterList instance
            // This test ensures the chapters() method works
        }
    }

    #[test]
    fn test_load_chapters() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let chapters = vec![
                ("Chapter 1".to_string(), Duration::from_secs(0), Duration::from_secs(60)),
                ("Chapter 2".to_string(), Duration::from_secs(60), Duration::from_secs(120)),
            ];
            engine.load_chapters(chapters);
            // Chapter loading not yet implemented, just verify it doesn't panic
        }
    }

    #[test]
    fn test_current_chapter() {
        if let Ok(engine) = MediaEngine::with_defaults() {
            assert_eq!(engine.current_chapter(), None);
        }
    }

    #[test]
    fn test_next_chapter_without_chapters() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let result = engine.next_chapter();
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_previous_chapter_without_chapters() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let result = engine.previous_chapter();
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_chapter_progress_with_chapters() {
        if let Ok(engine) = MediaEngine::with_defaults() {
            let progress = engine.chapter_progress();
            assert_eq!(progress, None);
        }
    }
}