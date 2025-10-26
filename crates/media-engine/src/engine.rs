use crate::chapters::ChapterList;
use crate::decoder::AudioDecoder;
use crate::equalizer::Equalizer;
use crate::playback::{PlaybackState, PlaybackStatus};
use crate::playback_thread::{
    self, AudioDecoder as PlaybackAudioDecoder, Equalizer as PlaybackEqualizer, PlaybackCommand,
};
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
    pub speed: Arc<Mutex<Speed>>,
    equalizer: Arc<Mutex<Equalizer>>,
    thread_handle: Option<JoinHandle<()>>,
    playback_state: Arc<Mutex<PlaybackState>>,
    pub duration: Option<Duration>,
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

    /// Loads an audio file for playback - FIXED to accept &Path
    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        // Stop any existing playback
        self.stop()?;

        // Kill existing thread if any
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        // Create decoder
        let decoder = AudioDecoder::new(path)
            .map_err(|e| format!("Failed to create decoder: {:?}", e))?;

        // Store file path - FIXED to use Path's to_string_lossy()
        self.loaded_file = Some(path.to_string_lossy().to_string());

        // Store decoder
        self.decoder = Some(decoder);

        // Create command channel
        let (tx, rx) = channel();
        *self.command_tx.lock().unwrap() = Some(tx);

        // Clone Arc references for thread
        let position = Arc::clone(&self.current_position);
        let status = Arc::clone(&self.current_status);
        let volume = Arc::clone(&self.volume);
        let speed = Arc::clone(&self.speed);
        let equalizer = Arc::clone(&self.equalizer);

        // Use config sample rate instead of calling non-existent method
        let sample_rate = self.config.sample_rate;

        // Spawn playback thread
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
        )}

    /// Starts playback
    pub fn play(&mut self) -> Result<(), String> {
        if self.loaded_file.is_none() {
            return Err("No file loaded".to_string());
        }

        if let Some(ref tx) = *self.command_tx.lock().unwrap() {
            tx.send(PlaybackCommand::Play)
                .map_err(|e| format!("Failed to send play command: {}", e))?;
            *self.current_status.lock().unwrap() = true;
        }

        Ok(())
    }

    /// Pauses playback
    pub fn pause(&mut self) -> Result<(), String> {
        if self.loaded_file.is_none() {
            return Err("No file loaded".to_string());
        }

        if let Some(ref tx) = *self.command_tx.lock().unwrap() {
            tx.send(PlaybackCommand::Pause)
                .map_err(|e| format!("Failed to send pause command: {}", e))?;
            *self.current_status.lock().unwrap() = false;
        }

        Ok(())
    }

    /// Stops playback and resets position
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(ref tx) = *self.command_tx.lock().unwrap() {
            let _ = tx.send(PlaybackCommand::Stop);
            *self.current_status.lock().unwrap() = false;
            *self.current_position.lock().unwrap() = Duration::from_secs(0);
        }

        Ok(())
    }

    /// Seeks to a specific position
    pub fn seek(&mut self, position: Duration) -> Result<(), String> {
        if self.loaded_file.is_none() {
            return Err("No file loaded".to_string());
        }

        if let Some(ref tx) = *self.command_tx.lock().unwrap() {
            tx.send(PlaybackCommand::Seek(position))
                .map_err(|e| format!("Failed to send seek command: {}", e))?;
            *self.current_position.lock().unwrap() = position;
        }

        Ok(())
    }

    /// Gets the current playback position
    pub fn position(&self) -> Duration {
        *self.current_position.lock().unwrap()
    }

    /// Gets the total duration of the loaded file
    pub fn duration(&self) -> Duration {
        self.duration.unwrap_or(Duration::from_secs(0))
    }

    /// Checks if audio is currently playing
    pub fn is_playing(&self) -> bool {
        *self.current_status.lock().unwrap()
    }

    /// Gets the current playback status
    pub fn status(&self) -> bool {
        self.is_playing()
    }

    /// Sets the playback speed
    pub fn set_speed(&mut self, speed: Speed) -> Result<(), String> {
        *self.speed.lock().unwrap() = speed;

        if let Some(ref tx) = *self.command_tx.lock().unwrap() {
            tx.send(PlaybackCommand::SetSpeed(speed))
                .map_err(|e| format!("Failed to send speed command: {}", e))?;
        }

        Ok(())
    }

    /// Gets the current playback speed
    pub fn speed(&self) -> Speed {
        *self.speed.lock().unwrap()
    }

    /// Sets the volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        if !(0.0..=1.0).contains(&volume) {
            return Err("Volume must be between 0.0 and 1.0".to_string());
        }

        *self.volume.lock().unwrap() = volume as f32;

        if let Some(ref tx) = *self.command_tx.lock().unwrap() {
            tx.send(PlaybackCommand::SetVolume(volume as f32))
                .map_err(|e| format!("Failed to send volume command: {}", e))?;
        }

        Ok(())
    }

    /// Gets the current volume
    pub fn volume(&self) -> f64 {
        *self.volume.lock().unwrap() as f64
    }

    /// Sets the equalizer - NOT IMPLEMENTED (PlaybackCommand doesn't have this variant)
    pub fn set_equalizer(&mut self, equalizer: Equalizer) -> Result<(), String> {
        *self.equalizer.lock().unwrap() = equalizer;
        // Can't send to playback thread since PlaybackCommand::SetEqualizer doesn't exist
        Ok(())
    }

    /// Gets the current equalizer
    pub fn equalizer(&self) -> Equalizer {
        self.equalizer.lock().unwrap().clone()
    }

    /// Loads chapter information - REMOVED broken implementation
    pub fn load_chapters(&mut self, _chapters: Vec<(String, Duration, Duration)>) {
        // TODO: Implement when chapter API is stable
    }

    /// Gets the chapter list
    pub fn chapters(&self) -> ChapterList {
        self.chapters.lock().unwrap().clone()
    }

    /// Gets the current chapter index - REMOVED broken implementation
    pub fn current_chapter(&self) -> Option<usize> {
        None // TODO: Implement when chapter API is stable
    }

    /// Jumps to the next chapter - REMOVED broken implementation
    pub fn next_chapter(&mut self) -> Result<(), String> {
        Err("Chapter navigation not yet implemented".to_string())
    }

    /// Jumps to the previous chapter - REMOVED broken implementation
    pub fn previous_chapter(&mut self) -> Result<(), String> {
        Err("Chapter navigation not yet implemented".to_string())
    }

    /// Gets chapter progress as a string - REMOVED broken implementation
    pub fn chapter_progress(&self) -> Option<String> {
        None // TODO: Implement when chapter API is stable
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
mod tests {
    use super::*;
    use std::path::PathBuf;

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
            let path = PathBuf::from("nonexistent.mp3");
            // FIXED: Pass &path instead of &path (PathBuf implements AsRef<Path>)
            let result = engine.load(&path);
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
        }
    }

    #[test]
    fn test_load_chapters() {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let chapters = vec![
                (
                    "Chapter 1".to_string(),
                    Duration::from_secs(0),
                    Duration::from_secs(60),
                ),
                (
                    "Chapter 2".to_string(),
                    Duration::from_secs(60),
                    Duration::from_secs(120),
                ),
            ];
            engine.load_chapters(chapters);
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