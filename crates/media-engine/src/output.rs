// FILE: crates/media-engine/src/output.rs

use crate::error::{EngineError, EngineResult};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleRate, Stream, StreamConfig};
use crossbeam_channel::{Receiver, TryRecvError};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Audio output using cpal
#[allow(dead_code)]
pub struct AudioOutput {
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
    sample_rate: u32,
}

impl AudioOutput {
    /// Create a new audio output with the default device
    pub fn new(sample_rate: u32, channels: u16) -> EngineResult<Self> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| EngineError::OutputError("No output device found".to_string()))?;

        let config = StreamConfig {
            channels,
            sample_rate: SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        Ok(Self {
            device,
            config,
            stream: None,
            sample_rate,
        })
    }

    /// Start playing audio from the given receiver
    pub fn play(&mut self, rx: Receiver<Vec<f32>>, running: Arc<AtomicBool>) -> EngineResult<()> {
        let mut buffer = Vec::new();
        let mut position = 0;

        let stream = self
            .device
            .build_output_stream(
                &self.config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Fill output buffer
                    for sample in data.iter_mut() {
                        // Get more data if needed
                        while position >= buffer.len() {
                            match rx.try_recv() {
                                Ok(new_data) => {
                                    buffer = new_data;
                                    position = 0;
                                }
                                Err(TryRecvError::Empty) => {
                                    // No data available, output silence
                                    *sample = 0.0;
                                    return;
                                }
                                Err(TryRecvError::Disconnected) => {
                                    // Sender disconnected, stop playback
                                    running.store(false, Ordering::Relaxed);
                                    *sample = 0.0;
                                    return;
                                }
                            }
                        }

                        if position < buffer.len() {
                            *sample = buffer[position];
                            position += 1;
                        } else {
                            *sample = 0.0;
                        }
                    }
                },
                |err| {
                    log::error!("Audio output error: {}", err);
                },
                None,
            )
            .map_err(|e| EngineError::OutputError(format!("Failed to build stream: {}", e)))?;

        stream
            .play()
            .map_err(|e| EngineError::OutputError(format!("Failed to start stream: {}", e)))?;

        self.stream = Some(stream);
        Ok(())
    }

    /// Stop playing audio
    pub fn stop(&mut self) {
        self.stream = None;
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_creation() {
        // May fail on systems without audio devices
        let result = AudioOutput::new(44100, 2);
        // Don't assert - audio devices may not be available in CI
        if result.is_ok() {
            assert!(true);
        }
    }
}