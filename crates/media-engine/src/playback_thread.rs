use crate::decoder::AudioDecoder;
use crate::equalizer::Equalizer;
use crate::playback::{PlaybackState, PlaybackStatus};
use crate::speed::Speed;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Commands that can be sent to the playback thread
#[derive(Debug, Clone)]
pub enum PlaybackCommand {
    Play,
    Pause,
    Stop,
    Seek(Duration),
    SetVolume(f32),
    SetSpeed(Speed),
}

/// Starts the playback thread
pub fn start_playback_thread(
    decoder: AudioDecoder,
    command_rx: Receiver<PlaybackCommand>,
    duration: Duration,
    current_position: Arc<Mutex<Duration>>,
    current_status: Arc<Mutex<bool>>,
    playback_state: Arc<Mutex<PlaybackState>>,
    volume: Arc<Mutex<f32>>,
    speed: Arc<Mutex<Speed>>,
    equalizer: Arc<Mutex<Equalizer>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut is_playing = false;

        // Update initial state
        if let Ok(mut state) = playback_state.lock() {
            state.set_duration(duration);
            state.set_status(PlaybackStatus::Stopped);
        }

        loop {
            // Check for commands
            if let Ok(command) = command_rx.try_recv() {
                match command {
                    PlaybackCommand::Play => {
                        is_playing = true;
                        if let Ok(mut state) = playback_state.lock() {
                            state.set_status(PlaybackStatus::Playing);
                        }
                    }
                    PlaybackCommand::Pause => {
                        is_playing = false;
                        if let Ok(mut state) = playback_state.lock() {
                            state.set_status(PlaybackStatus::Paused);
                        }
                    }
                    PlaybackCommand::Stop => {
                        is_playing = false;
                        if let Ok(mut state) = playback_state.lock() {
                            *state = PlaybackState::stopped();
                        }
                        break;
                    }
                    PlaybackCommand::Seek(position) => {
                        if let Ok(mut pos) = current_position.lock() {
                            *pos = position;
                        }
                        if let Ok(mut state) = playback_state.lock() {
                            state.set_position(position);
                        }
                    }
                    PlaybackCommand::SetVolume(_vol) => {
                        // Volume is handled via Arc<Mutex<f32>>
                    }
                    PlaybackCommand::SetSpeed(_spd) => {
                        // Speed is handled via Arc<Mutex<Speed>>
                    }
                }
            }

            if is_playing {
                // Simulate playback and update position
                thread::sleep(Duration::from_millis(10));

                if let Ok(mut pos) = current_position.lock() {
                    *pos += Duration::from_millis(10);

                    // Update playback state
                    if let Ok(mut state) = playback_state.lock() {
                        state.set_position(*pos);
                    }

                    // Check if we've reached the end
                    if *pos >= duration {
                        is_playing = false;
                        if let Ok(mut state) = playback_state.lock() {
                            state.set_status(PlaybackStatus::Stopped);
                        }
                        if let Ok(mut status) = current_status.lock() {
                            *status = false;
                        }
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(50));
            }
        }

        // Ensure resources are properly cleaned up
        drop(decoder);
        drop(volume);
        drop(speed);
        drop(equalizer);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_command_clone() {
        let cmd = PlaybackCommand::Play;
        let _cloned = cmd.clone();
    }

    #[test]
    fn test_playback_thread_nonexistent() {
        // This test just ensures the module compiles
        // Actual playback thread testing requires real audio files
    }
}