// crates/media-engine/src/playback_thread.rs

use crate::output::AudioOutput;
use crate::playback::{PlaybackState, PlaybackStatus};
use crate::speed::{Speed, SpeedProcessor};
use crossbeam_channel::{bounded, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

/// Commands that can be sent to the playback thread
#[derive(Debug, Clone)]
pub enum PlaybackCommand {
    Play,
    Pause,
    Stop,
    Seek(std::time::Duration),
    SetVolume(f32),
    SetSpeed(Speed),
}

/// Audio processing pipeline state
struct AudioPipeline {
    decoder: AudioDecoder,
    speed_processor: SpeedProcessor,
    equalizer: Equalizer,
    output: AudioOutput,
    volume: f32,
    is_playing: bool,
    running: Arc<AtomicBool>,
}

impl AudioPipeline {
    fn new(decoder: AudioDecoder, sample_rate: u32, channels: u16) -> Result<Self, String> {
        let speed_processor = SpeedProcessor::new(sample_rate, channels);
        let equalizer = Equalizer::default();
        let output = AudioOutput::new(sample_rate, channels)
            .map_err(|e| format!("Failed to create audio output: {}", e))?;

        Ok(Self {
            decoder,
            speed_processor,
            equalizer,
            output,
            volume: 1.0,
            is_playing: false,
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    fn process_audio_chunk(&mut self, tx: &Sender<Vec<f32>>) -> Result<bool, String> {
        // Decode a chunk of audio
        const CHUNK_SIZE: usize = 4096;
        let decoded = match self.decoder.decode_chunk(CHUNK_SIZE) {
            Ok(samples) if !samples.is_empty() => samples,
            Ok(_) => return Ok(false), // End of file
            Err(e) => return Err(format!("Decode error: {}", e)),
        };

        // Process through speed adjustment
        let speed_adjusted = self
            .speed_processor
            .process(&decoded)
            .map_err(|e| format!("Speed processing error: {}", e))?;

        // Apply equalizer (for now just pass through since process method doesn't exist)
        let equalized = self.equalizer.apply(&speed_adjusted);

        // Apply volume
        let final_audio: Vec<f32> = equalized
            .into_iter()
            .map(|s| (s * self.volume).clamp(-1.0, 1.0))
            .collect();

        // Send to output
        tx.send(final_audio)
            .map_err(|_| "Failed to send audio to output".to_string())?;

        Ok(true)
    }

    fn seek(&mut self, position: Duration) -> Result<(), String> {
        self.decoder
            .seek(position)
            .map_err(|e| format!("Seek failed: {}", e))?;

        // Clear speed processor buffers after seeking
        self.speed_processor.reset();

        Ok(())
    }
}

/// Starts the playback thread with real audio processing
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
        // Get audio format info from decoder
        let (sample_rate, channels) = match decoder.get_format() {
            Ok(fmt) => fmt,
            Err(e) => {
                log::error!("Failed to get audio format: {}", e);
                return;
            }
        };

        // Create audio pipeline
        let mut pipeline = match AudioPipeline::new(decoder, sample_rate, channels as u16) {
            Ok(p) => p,
            Err(e) => {
                log::error!("Failed to create audio pipeline: {}", e);
                return;
            }
        };

        // Create channel for audio data
        let (audio_tx, audio_rx) = bounded::<Vec<f32>>(16); // Buffer up to 16 chunks

        // Start audio output stream
        let running = pipeline.running.clone();
        if let Err(e) = pipeline.output.play(audio_rx, running.clone()) {
            log::error!("Failed to start audio output: {}", e);
            return;
        }

        // Update initial state
        if let Ok(mut state) = playback_state.lock() {
            state.set_duration(duration);
            state.set_status(PlaybackStatus::Stopped);
        }

        let mut last_position_update = Instant::now();
        let mut accumulated_samples = 0u64;

        // Main playback loop
        while running.load(Ordering::Relaxed) {
            // Check for commands
            if let Ok(command) = command_rx.try_recv() {
                match command {
                    PlaybackCommand::Play => {
                        pipeline.is_playing = true;
                        if let Ok(mut state) = playback_state.lock() {
                            state.set_status(PlaybackStatus::Playing);
                        }
                        if let Ok(mut status) = current_status.lock() {
                            *status = true;
                        }
                    }
                    PlaybackCommand::Pause => {
                        pipeline.is_playing = false;
                        if let Ok(mut state) = playback_state.lock() {
                            state.set_status(PlaybackStatus::Paused);
                        }
                        if let Ok(mut status) = current_status.lock() {
                            *status = false;
                        }
                    }
                    PlaybackCommand::Stop => {
                        pipeline.is_playing = false;
                        if let Ok(mut state) = playback_state.lock() {
                            *state = PlaybackState::stopped();
                        }
                        running.store(false, Ordering::Relaxed);
                        break;
                    }
                    PlaybackCommand::Seek(position) => {
                        if let Err(e) = pipeline.seek(position) {
                            log::error!("Seek failed: {}", e);
                        } else {
                            accumulated_samples =
                                (position.as_secs_f64() * sample_rate as f64) as u64;
                            if let Ok(mut pos) = current_position.lock() {
                                *pos = position;
                            }
                            if let Ok(mut state) = playback_state.lock() {
                                state.set_position(position);
                            }
                        }
                    }
                    PlaybackCommand::SetVolume(vol) => {
                        pipeline.volume = vol;
                        if let Ok(mut v) = volume.lock() {
                            *v = vol;
                        }
                    }
                    PlaybackCommand::SetSpeed(new_speed) => {
                        if let Err(e) = pipeline.speed_processor.set_speed(new_speed) {
                            log::error!("Failed to set speed: {}", e);
                        } else if let Ok(mut s) = speed.lock() {
                            *s = new_speed;
                        }
                    }
                }
            }

            // Update equalizer settings
            if let Ok(eq) = equalizer.lock() {
                pipeline.equalizer = eq.clone();
            }

            // Process audio if playing
            if pipeline.is_playing {
                match pipeline.process_audio_chunk(&audio_tx) {
                    Ok(true) => {
                        // Successfully processed audio

                        // Update position based on actual samples processed
                        // Account for speed adjustment
                        let current_speed = speed.lock().map(|s| s.value()).unwrap_or(1.0);

                        accumulated_samples += (4096.0 / current_speed) as u64;

                        // Update position periodically (not every chunk for performance)
                        if last_position_update.elapsed() > Duration::from_millis(100) {
                            let new_position = Duration::from_secs_f64(
                                accumulated_samples as f64 / sample_rate as f64,
                            );

                            if let Ok(mut pos) = current_position.lock() {
                                *pos = new_position;
                            }
                            if let Ok(mut state) = playback_state.lock() {
                                state.set_position(new_position);
                            }

                            last_position_update = Instant::now();
                        }
                    }
                    Ok(false) => {
                        // End of file reached
                        log::info!("Playback completed");
                        pipeline.is_playing = false;

                        if let Ok(mut state) = playback_state.lock() {
                            state.set_status(PlaybackStatus::Stopped);
                            state.set_position(duration);
                        }
                        if let Ok(mut status) = current_status.lock() {
                            *status = false;
                        }
                    }
                    Err(e) => {
                        log::error!("Audio processing error: {}", e);
                        pipeline.is_playing = false;

                        if let Ok(mut state) = playback_state.lock() {
                            state.set_status(PlaybackStatus::Stopped);
                        }
                        if let Ok(mut status) = current_status.lock() {
                            *status = false;
                        }
                    }
                }
            } else {
                // When paused, send silence to avoid buffer underruns
                let silence = vec![0.0f32; 1024];
                let _ = audio_tx.try_send(silence);
                thread::sleep(Duration::from_millis(10));
            }
        }

        // Cleanup
        pipeline.output.stop();
        log::info!("Playback thread terminated");
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
    fn test_playback_command_variants() {
        let _play = PlaybackCommand::Play;
        let _pause = PlaybackCommand::Pause;
        let _stop = PlaybackCommand::Stop;
        let _seek = PlaybackCommand::Seek(Duration::from_secs(10));
        let _volume = PlaybackCommand::SetVolume(0.5);
        let _speed = PlaybackCommand::SetSpeed(Speed::default());
    }
}

// crates/media-engine/src/decoder.rs

use crate::error::{EngineError, EngineResult};
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_codecs;
use symphonia::default::get_probe;

/// Audio decoder using Symphonia
pub struct AudioDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_rate: u32,
    channels: usize,
    sample_buffer: Option<SampleBuffer<f32>>,
}

impl AudioDecoder {
    /// Create a new decoder for the given file
    pub fn new(path: &Path) -> EngineResult<Self> {
        let file = File::open(path)
            .map_err(|e| EngineError::DecodeError(format!("Failed to open file: {}", e)))?;

        let source = Box::new(file);
        let mss = MediaSourceStream::new(source, Default::default());

        // Create hint from file extension
        let mut hint = Hint::new();
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                hint.with_extension(ext_str);
            }
        }

        // Probe the media
        let probe = get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| EngineError::DecodeError(format!("Unsupported format: {}", e)))?;

        let format = probe.format;

        // Find the first audio track
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| EngineError::DecodeError("No audio tracks found".to_string()))?;

        let track_id = track.id;

        // Get audio parameters
        let sample_rate = track
            .codec_params
            .sample_rate
            .ok_or_else(|| EngineError::DecodeError("Unknown sample rate".to_string()))?;

        let channels = track
            .codec_params
            .channels
            .map(|ch| ch.count())
            .unwrap_or(2);

        // Create decoder
        let decoder = get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| EngineError::DecodeError(format!("Failed to create decoder: {}", e)))?;

        Ok(Self {
            format,
            decoder,
            track_id,
            sample_rate,
            channels,
            sample_buffer: None,
        })
    }

    /// Decode a chunk of audio samples
    pub fn decode_chunk(&mut self, max_samples: usize) -> EngineResult<Vec<f32>> {
        let mut output = Vec::with_capacity(max_samples);

        while output.len() < max_samples {
            // Get next packet
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::IoError(_)) => {
                    // End of stream
                    break;
                }
                Err(e) => {
                    return Err(EngineError::DecodeError(format!(
                        "Packet read error: {}",
                        e
                    )));
                }
            };

            // Skip packets from other tracks
            if packet.track_id() != self.track_id {
                continue;
            }

            // Decode packet
            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(symphonia::core::errors::Error::DecodeError(_)) => {
                    // Skip decode errors
                    continue;
                }
                Err(e) => {
                    return Err(EngineError::DecodeError(format!("Decode error: {}", e)));
                }
            };

            // Convert to f32 samples
            let buffer_spec = *decoded.spec();
            let buffer_duration = decoded.frames() as usize;
            let buffer_channels = buffer_spec.channels.count();

            // Create or resize sample buffer
            if self.sample_buffer.is_none()
                || self.sample_buffer.as_ref().unwrap().len() != buffer_duration * buffer_channels
            {
                self.sample_buffer = Some(SampleBuffer::new(buffer_duration as u64, buffer_spec));
            }

            // Copy to sample buffer
            if let Some(ref mut sample_buf) = self.sample_buffer {
                sample_buf.copy_interleaved_ref(decoded);
                output.extend_from_slice(sample_buf.samples());
            } else {
                return Err(EngineError::DecodeError(
                    "Failed to create sample buffer".to_string(),
                ));
            }
        }

        // Truncate to requested size
        output.truncate(max_samples);
        Ok(output)
    }

    /// Convert audio buffer to f32 samples
    #[allow(dead_code)]
    fn audio_buffer_to_samples(&mut self, audio_buf: AudioBufferRef) -> EngineResult<Vec<f32>> {
        let spec = *audio_buf.spec();
        let duration = audio_buf.frames() as usize;
        let channels = spec.channels.count();

        // Create or resize sample buffer
        if self.sample_buffer.is_none()
            || self.sample_buffer.as_ref().unwrap().len() != duration * channels
        {
            self.sample_buffer = Some(SampleBuffer::new(duration as u64, spec));
        }

        // Copy to sample buffer
        if let Some(ref mut sample_buf) = self.sample_buffer {
            sample_buf.copy_interleaved_ref(audio_buf);
            Ok(sample_buf.samples().to_vec())
        } else {
            Err(EngineError::DecodeError(
                "Failed to create sample buffer".to_string(),
            ))
        }
    }

    /// Seek to a specific position
    pub fn seek(&mut self, position: Duration) -> EngineResult<()> {
        let seek_time = position.as_secs();

        self.format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time: symphonia::core::units::Time::from(seek_time),
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| EngineError::DecodeError(format!("Seek failed: {}", e)))?;

        // Reset decoder after seek
        self.decoder.reset();

        Ok(())
    }

    /// Get the audio format (sample rate, channels)
    pub fn get_format(&self) -> EngineResult<(u32, usize)> {
        Ok((self.sample_rate, self.channels))
    }

    /// Get the duration of the audio file
    pub fn duration(&self) -> Option<Duration> {
        let track = self
            .format
            .tracks()
            .iter()
            .find(|t| t.id == self.track_id)?;

        let time_base = track.codec_params.time_base?;
        let n_frames = track.codec_params.n_frames?;

        let duration_secs = time_base.calc_time(n_frames).seconds as f64;
        Some(Duration::from_secs_f64(duration_secs))
    }
}

#[cfg(test)]
mod decoder_tests {
    use super::*;

    #[test]
    fn test_unsupported_file_error() {
        let result = AudioDecoder::new(Path::new("nonexistent.xyz"));
        assert!(result.is_err());
    }
}

// crates/media-engine/src/equalizer.rs
// Add the apply method to the existing Equalizer implementation

// UNUSED IMPORT - WILL BE USED LATER
// use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct EqualizerBand {
    pub frequency: f32,
    pub gain: f32,
    pub q_factor: f32,
}

#[derive(Debug, Clone)]
pub struct Equalizer {
    bands: Vec<EqualizerBand>,
    enabled: bool,
}

impl Default for Equalizer {
    fn default() -> Self {
        Self {
            bands: Self::default_bands(),
            enabled: false,
        }
    }
}

impl Equalizer {
    fn default_bands() -> Vec<EqualizerBand> {
        vec![
            EqualizerBand {
                frequency: 32.0,
                gain: 0.0,
                q_factor: 0.7,
            },
            EqualizerBand {
                frequency: 64.0,
                gain: 0.0,
                q_factor: 0.7,
            },
            EqualizerBand {
                frequency: 125.0,
                gain: 0.0,
                q_factor: 0.7,
            },
            EqualizerBand {
                frequency: 250.0,
                gain: 0.0,
                q_factor: 0.7,
            },
            EqualizerBand {
                frequency: 500.0,
                gain: 0.0,
                q_factor: 0.7,
            },
            EqualizerBand {
                frequency: 1000.0,
                gain: 0.0,
                q_factor: 0.7,
            },
            EqualizerBand {
                frequency: 2000.0,
                gain: 0.0,
                q_factor: 0.7,
            },
            EqualizerBand {
                frequency: 4000.0,
                gain: 0.0,
                q_factor: 0.7,
            },
            EqualizerBand {
                frequency: 8000.0,
                gain: 0.0,
                q_factor: 0.7,
            },
            EqualizerBand {
                frequency: 16000.0,
                gain: 0.0,
                q_factor: 0.7,
            },
        ]
    }

    /// Apply equalizer to audio samples (passthrough for now)
    pub fn apply(&self, samples: &[f32]) -> Vec<f32> {
        if !self.enabled {
            return samples.to_vec();
        }

        // For now, just pass through the audio
        // Full EQ implementation would require FFT processing
        samples.to_vec()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_band_gain(&mut self, band_index: usize, gain: f32) {
        if let Some(band) = self.bands.get_mut(band_index) {
            band.gain = gain.clamp(-12.0, 12.0);
        }
    }

    pub fn reset(&mut self) {
        self.bands = Self::default_bands();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EqualizerPreset {
    Flat,
    Rock,
    Jazz,
    Classical,
    Pop,
    Bass,
    Treble,
    Vocal,
    Custom,
}

impl EqualizerPreset {
    pub fn apply_to(&self, eq: &mut Equalizer) {
        let gains = match self {
            Self::Flat => vec![0.0; 10],
            Self::Rock => vec![5.0, 4.0, 3.0, 1.0, -1.0, -1.0, 1.0, 3.0, 4.0, 5.0],
            Self::Jazz => vec![4.0, 3.0, 1.0, 2.0, -2.0, -2.0, 0.0, 1.0, 3.0, 4.0],
            Self::Classical => vec![-2.0, -2.0, -2.0, -1.0, 2.0, 3.0, 3.0, 2.0, 0.0, -1.0],
            Self::Pop => vec![-2.0, -1.0, 2.0, 4.0, 5.0, 4.0, 2.0, 0.0, -1.0, -2.0],
            Self::Bass => vec![6.0, 5.0, 4.0, 3.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Self::Treble => vec![-2.0, -2.0, -1.0, 0.0, 1.0, 2.0, 4.0, 5.0, 6.0, 7.0],
            Self::Vocal => vec![-2.0, -3.0, -3.0, 1.0, 4.0, 4.0, 3.0, 1.0, 0.0, -1.0],
            Self::Custom => return, // Don't change anything for custom
        };

        for (i, &gain) in gains.iter().enumerate() {
            eq.set_band_gain(i, gain);
        }
    }
}
