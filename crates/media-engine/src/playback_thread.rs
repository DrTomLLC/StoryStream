// FILE: crates/media-engine/src/playback_thread.rs

use crate::decoder::AudioDecoder;
use crate::error::{EngineError, EngineResult};
use crate::output::AudioOutput;
use crossbeam_channel::{Sender, bounded};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration as StdDuration;

/// Commands sent to the playback thread
#[derive(Debug, Clone)]
pub enum PlaybackCommand {
    Play,
    Pause,
    Seek(f64),
    SetVolume(f32),
    SetSpeed(f32),
    Stop,
}

/// Playback thread handle
pub struct PlaybackThread {
    handle: Option<thread::JoinHandle<()>>,
    command_tx: Sender<PlaybackCommand>,
    running: Arc<AtomicBool>,
    position: Arc<AtomicU64>,
}

impl PlaybackThread {
    /// Start a new playback thread for the given file
    pub fn start(path: &Path) -> EngineResult<Self> {
        // Validate file exists before starting thread
        if !path.exists() {
            return Err(EngineError::DecodeError(format!(
                "File not found: {}",
                path.display()
            )));
        }
        
        let path = path.to_path_buf();
        let running = Arc::new(AtomicBool::new(true));
        let position = Arc::new(AtomicU64::new(0));
        let (command_tx, command_rx) = bounded(10);

        let running_clone = Arc::clone(&running);
        let position_clone = Arc::clone(&position);

        let handle = thread::spawn(move || {
            if let Err(e) = playback_loop(&path, command_rx, running_clone, position_clone) {
                log::error!("Playback thread error: {}", e);
            }
        });

        Ok(Self {
            handle: Some(handle),
            command_tx,
            running,
            position,
        })
    }

    /// Send a command to the playback thread
    pub fn send_command(&self, cmd: PlaybackCommand) -> EngineResult<()> {
        self.command_tx
            .send(cmd)
            .map_err(|e| EngineError::InvalidState(format!("Failed to send command: {}", e)))
    }

    /// Get the current playback position in seconds
    pub fn position(&self) -> f64 {
        f64::from_bits(self.position.load(Ordering::Relaxed))
    }

    /// Check if the thread is still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop the playback thread
    pub fn stop(&mut self) {
        let _ = self.send_command(PlaybackCommand::Stop);
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for PlaybackThread {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The main playback loop
fn playback_loop(
    path: &Path,
    command_rx: crossbeam_channel::Receiver<PlaybackCommand>,
    running: Arc<AtomicBool>,
    position: Arc<AtomicU64>,
) -> EngineResult<()> {
    // Create decoder
    let mut decoder = AudioDecoder::new(path)?;
    let spec = *decoder.spec();

    // Create audio output
    let (audio_tx, audio_rx) = bounded(4);
    let mut output = AudioOutput::new(spec.rate, spec.channels.count() as u16)?;

    let running_clone = Arc::clone(&running);
    output.play(audio_rx, running_clone)?;

    let mut playing = false;
    let mut volume = 1.0_f32;
    let mut current_position = 0.0_f64;

    while running.load(Ordering::Relaxed) {
        // Check for commands
        match command_rx.try_recv() {
            Ok(PlaybackCommand::Play) => {
                playing = true;
            }
            Ok(PlaybackCommand::Pause) => {
                playing = false;
            }
            Ok(PlaybackCommand::Seek(time)) => {
                decoder.seek(time)?;
                current_position = time;
                position.store(time.to_bits(), Ordering::Relaxed);
            }
            Ok(PlaybackCommand::SetVolume(vol)) => {
                volume = vol.clamp(0.0, 1.0);
            }
            Ok(PlaybackCommand::SetSpeed(_speed)) => {
                // TODO: Implement speed change with rubato resampler
                log::warn!("Speed change not yet implemented");
            }
            Ok(PlaybackCommand::Stop) => {
                break;
            }
            Err(_) => {
                // No command, continue
            }
        }

        if !playing {
            thread::sleep(StdDuration::from_millis(10));
            continue;
        }

        // Decode next packet
        match decoder.decode_next()? {
            Some(mut decoded) => {
                // Apply volume
                for sample in &mut decoded.samples {
                    *sample *= volume;
                }

                // Update position
                let frames = decoded.samples.len() / decoded.spec.channels.count();
                let time_delta = frames as f64 / decoded.spec.rate as f64;
                current_position += time_delta;
                position.store(current_position.to_bits(), Ordering::Relaxed);

                // Send to output
                if audio_tx.send(decoded.samples).is_err() {
                    break;
                }
            }
            None => {
                // End of file
                playing = false;
                log::info!("Playback finished");
            }
        }
    }

    output.stop();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_command_clone() {
        let cmd = PlaybackCommand::Play;
        let _ = cmd.clone();
    }

    #[test]
    fn test_playback_thread_nonexistent() {
        let result = PlaybackThread::start(Path::new("nonexistent.mp3"));
        assert!(result.is_err());
    }
}