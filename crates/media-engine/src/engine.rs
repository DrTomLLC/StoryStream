use crate::error::{MediaError, Result};
use crate::types::{Chapter, MediaEvent, PlaybackState, PlaybackStatus};

use crossbeam_channel::{Receiver, Sender};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Main media playback engine
pub struct MediaEngine {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Option<Sink>,
    state: Arc<RwLock<PlaybackState>>,
    chapters: Vec<Chapter>,
    current_file: Option<PathBuf>,
    event_tx: Sender<MediaEvent>,
    event_rx: Receiver<MediaEvent>,
}

impl MediaEngine {
    /// Create a new media engine
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| MediaError::PlaybackError(format!("Failed to create audio output: {}", e)))?;

        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        info!("Media engine initialized");

        Ok(Self {
            _stream: stream,
            stream_handle,
            sink: None,
            state: Arc::new(RwLock::new(PlaybackState::default())),
            chapters: Vec::new(),
            current_file: None,
            event_tx,
            event_rx,
        })
    }

    /// Load a media file
    pub fn load<P: AsRef<Path>>(&mut self, path: P, book_id: Option<i64>) -> Result<()> {
        let path = path.as_ref();
        info!("Loading media: {:?}", path);

        self.stop();

        let file = File::open(path)
            .map_err(|e| MediaError::LoadError(format!("Failed to open file: {}", e)))?;

        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| MediaError::DecodeError(format!("Failed to decode: {}", e)))?;

        let duration = source
            .total_duration()
            .ok_or_else(|| MediaError::LoadError("Could not determine duration".to_string()))?;

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| MediaError::PlaybackError(format!("Failed to create sink: {}", e)))?;

        sink.append(source);
        sink.pause();

        self.sink = Some(sink);
        self.current_file = Some(path.to_path_buf());

        {
            let mut state = self.state.write().unwrap();
            state.duration = duration;
            state.position = Duration::ZERO;
            state.status = PlaybackStatus::Stopped;
            state.book_id = book_id;
            state.current_chapter = None;
        }

        self.emit_event(MediaEvent::StateChanged(self.get_state()));

        info!("Media loaded: {:?}", duration);
        Ok(())
    }

    /// Start or resume playback
    pub fn play(&mut self) -> Result<()> {
        let sink = self.sink.as_ref().ok_or(MediaError::NoMediaLoaded)?;

        if sink.is_paused() {
            sink.play();
            self.update_status(PlaybackStatus::Playing);
            info!("Playback started");
            Ok(())
        } else {
            warn!("Already playing");
            Ok(())
        }
    }

    /// Pause playback
    pub fn pause(&mut self) -> Result<()> {
        let sink = self.sink.as_ref().ok_or(MediaError::NoMediaLoaded)?;

        if !sink.is_paused() {
            sink.pause();
            self.update_status(PlaybackStatus::Paused);
            info!("Playback paused");
        }
        Ok(())
    }

    /// Stop playback
    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
            info!("Playback stopped");
        }

        self.update_status(PlaybackStatus::Stopped);
        self.update_position(Duration::ZERO);
    }

    /// Toggle play/pause
    pub fn toggle_playback(&mut self) -> Result<()> {
        match self.get_state().status {
            PlaybackStatus::Playing => self.pause(),
            PlaybackStatus::Paused | PlaybackStatus::Stopped => self.play(),
            PlaybackStatus::Buffering => Ok(()),
        }
    }

    /// Seek to position
    pub fn seek(&mut self, position: Duration) -> Result<()> {
        let duration = self.get_state().duration;

        if position > duration {
            return Err(MediaError::InvalidSeek(format!(
                "Position {:?} exceeds duration {:?}",
                position, duration
            )));
        }

        let file = self
            .current_file
            .clone()
            .ok_or(MediaError::NoMediaLoaded)?;

        let book_id = self.get_state().book_id;
        let was_playing = self.get_state().is_playing();

        self.load(&file, book_id)?;
        self.update_position(position);

        if was_playing {
            self.play()?;
        }

        self.emit_event(MediaEvent::PositionChanged(position));
        self.update_current_chapter();

        info!("Seeked to {:?}", position);
        Ok(())
    }

    /// Skip forward
    pub fn skip_forward(&mut self, duration: Duration) -> Result<()> {
        let current = self.get_state().position;
        self.seek(current + duration)
    }

    /// Skip backward
    pub fn skip_backward(&mut self, duration: Duration) -> Result<()> {
        let current = self.get_state().position;
        let new_pos = current.saturating_sub(duration);
        self.seek(new_pos)
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f32) -> Result<()> {
        if !(0.25..=3.0).contains(&speed) {
            return Err(MediaError::InvalidSpeed(speed));
        }

        if let Some(sink) = &self.sink {
            sink.set_speed(speed);
            let mut state = self.state.write().unwrap();
            state.speed = speed;
            info!("Speed set to {}x", speed);
        }

        Ok(())
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        let volume = volume.clamp(0.0, 1.0);

        if let Some(sink) = &self.sink {
            sink.set_volume(volume);
            let mut state = self.state.write().unwrap();
            state.volume = volume;
            debug!("Volume set to {}", volume);
        }

        Ok(())
    }

    /// Load chapters
    pub fn load_chapters(&mut self, chapters: Vec<Chapter>) {
        let mut sorted = chapters;
        sorted.sort_by_key(|c| c.start_time);
        self.chapters = sorted;
        self.update_current_chapter();
        info!("Loaded {} chapters", self.chapters.len());
    }

    /// Jump to chapter
    /// Jump to chapter
    pub fn jump_to_chapter(&mut self, index: usize) -> Result<()> {
        let chapter = self
            .chapters
            .get(index)
            .ok_or(MediaError::ChapterNotFound(index))?;

        // Clone what we need before calling seek (to avoid borrow conflict)
        let start_time = chapter.start_time;
        let title = chapter.title.clone();

        self.seek(start_time)?;
        self.emit_event(MediaEvent::ChapterChanged(index));
        info!("Jumped to chapter {}: {}", index, title);
        Ok(())
    }

    /// Next chapter
    pub fn next_chapter(&mut self) -> Result<()> {
        let current_pos = self.get_state().position;
        let next = self
            .chapters
            .iter()
            .find(|c| c.start_time > current_pos)
            .ok_or(MediaError::ChapterNotFound(usize::MAX))?;

        self.jump_to_chapter(next.index)
    }

    /// Previous chapter
    pub fn previous_chapter(&mut self) -> Result<()> {
        let current_pos = self.get_state().position;

        if let Some(current) = self.current_chapter() {
            if current_pos - current.start_time > Duration::from_secs(3) {
                return self.jump_to_chapter(current.index);
            }
        }

        let prev = self
            .chapters
            .iter()
            .rev()
            .find(|c| c.start_time < current_pos)
            .ok_or(MediaError::ChapterNotFound(0))?;

        self.jump_to_chapter(prev.index)
    }

    /// Get current chapter
    pub fn current_chapter(&self) -> Option<&Chapter> {
        let position = self.get_state().position;
        self.chapters.iter().find(|c| c.contains(position))
    }

    /// Get all chapters
    pub fn chapters(&self) -> &[Chapter] {
        &self.chapters
    }

    /// Get current state
    pub fn get_state(&self) -> PlaybackState {
        self.state.read().unwrap().clone()
    }

    /// Check if media is loaded
    pub fn is_loaded(&self) -> bool {
        self.sink.is_some()
    }

    /// Get event receiver
    pub fn events(&self) -> Receiver<MediaEvent> {
        self.event_rx.clone()
    }

    fn update_status(&self, status: PlaybackStatus) {
        let mut state = self.state.write().unwrap();
        state.status = status;
        self.emit_event(MediaEvent::StateChanged(state.clone()));
    }

    fn update_position(&self, position: Duration) {
        let mut state = self.state.write().unwrap();
        state.position = position;
    }

    fn update_current_chapter(&self) {
        let position = self.get_state().position;
        let chapter_index = self
            .chapters
            .iter()
            .position(|c| c.contains(position));

        let mut state = self.state.write().unwrap();
        if state.current_chapter != chapter_index {
            state.current_chapter = chapter_index;
            if let Some(idx) = chapter_index {
                self.emit_event(MediaEvent::ChapterChanged(idx));
            }
        }
    }

    fn emit_event(&self, event: MediaEvent) {
        let _ = self.event_tx.send(event);
    }
}

impl Drop for MediaEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = MediaEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_no_audio_loaded_error() {
        let mut engine = MediaEngine::new().unwrap();
        assert!(engine.play().is_err());
        assert!(engine.pause().is_err());
    }

    #[test]
    fn test_speed_validation() {
        let mut engine = MediaEngine::new().unwrap();
        assert!(engine.set_speed(0.5).is_ok());
        assert!(engine.set_speed(2.0).is_ok());
        assert!(engine.set_speed(0.1).is_err());
        assert!(engine.set_speed(5.0).is_err());
    }
}