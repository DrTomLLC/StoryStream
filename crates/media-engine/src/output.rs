// crates/media-engine/src/output.rs
// Enhanced audio output with device selection

use crate::audio_device::{AudioDeviceInfo, AudioDeviceManager};
use crate::error::{EngineError, EngineResult};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, SampleRate, Stream, StreamConfig};
use crossbeam_channel::{Receiver, TryRecvError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Audio output configuration
#[derive(Debug, Clone)]
pub struct AudioOutputConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub device_id: Option<String>,
    pub buffer_size: Option<u32>,
}

impl Default for AudioOutputConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            device_id: None,
            buffer_size: None,
        }
    }
}

/// Enhanced audio output with device selection
pub struct AudioOutput {
    device: Device,
    device_info: AudioDeviceInfo,
    config: StreamConfig,
    stream: Option<Stream>,
    sample_rate: u32,
    manager: AudioDeviceManager,
}

impl AudioOutput {
    /// Create audio output with specific configuration
    pub fn with_config(config: AudioOutputConfig) -> EngineResult<Self> {
        let mut manager = AudioDeviceManager::new()?;

        // Select device
        if let Some(device_id) = &config.device_id {
            manager.select_device(device_id)?;
        } else {
            manager.select_default_device()?;
        }

        let device = manager.get_output_device()?;
        let device_info = manager
            .get_selected_device()
            .ok_or_else(|| EngineError::OutputError("No device selected".to_string()))?
            .clone();

        // Validate configuration against device capabilities
        if config.channels < device_info.min_channels || config.channels > device_info.max_channels
        {
            return Err(EngineError::OutputError(format!(
                "Device {} supports {}-{} channels, requested {}",
                device_info.name,
                device_info.min_channels,
                device_info.max_channels,
                config.channels
            )));
        }

        if !device_info.sample_rates.is_empty()
            && !device_info.sample_rates.contains(&config.sample_rate)
        {
            log::warn!(
                "Sample rate {} may not be optimal for device {}",
                config.sample_rate,
                device_info.name
            );
        }

        let stream_config = StreamConfig {
            channels: config.channels,
            sample_rate: SampleRate(config.sample_rate),
            buffer_size: config
                .buffer_size
                .map(|s| cpal::BufferSize::Fixed(s))
                .unwrap_or(cpal::BufferSize::Default),
        };

        Ok(Self {
            device,
            device_info,
            config: stream_config,
            stream: None,
            sample_rate: config.sample_rate,
            manager,
        })
    }

    /// Create audio output with default device and settings
    pub fn new(sample_rate: u32, channels: u16) -> EngineResult<Self> {
        Self::with_config(AudioOutputConfig {
            sample_rate,
            channels,
            ..Default::default()
        })
    }

    /// List available audio devices
    pub fn list_devices(&self) -> Vec<AudioDeviceInfo> {
        self.manager.list_devices()
    }

    /// Get current device info
    pub fn device_info(&self) -> &AudioDeviceInfo {
        &self.device_info
    }

    /// Check if current device is still available
    pub fn is_device_available(&self) -> bool {
        self.manager.is_device_available(&self.device_info.id)
    }

    /// Start playing audio from the given receiver
    pub fn play(&mut self, rx: Receiver<Vec<f32>>, running: Arc<AtomicBool>) -> EngineResult<()> {
        // Check device is still available
        if !self.is_device_available() {
            return Err(EngineError::OutputError(format!(
                "Audio device '{}' is no longer available",
                self.device_info.name
            )));
        }

        let mut buffer = Vec::new();
        let mut position = 0;

        let device_name = self.device_info.name.clone();

        let stream = self
            .device
            .build_output_stream(
                &self.config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for sample in data.iter_mut() {
                        while position >= buffer.len() {
                            match rx.try_recv() {
                                Ok(new_data) => {
                                    buffer = new_data;
                                    position = 0;
                                }
                                Err(TryRecvError::Empty) => {
                                    *sample = 0.0;
                                    return;
                                }
                                Err(TryRecvError::Disconnected) => {
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
                move |err| {
                    log::error!("Audio output error on device '{}': {}", device_name, err);
                },
                None,
            )
            .map_err(|e| EngineError::OutputError(format!("Failed to build stream: {}", e)))?;

        stream
            .play()
            .map_err(|e| EngineError::OutputError(format!("Failed to start stream: {}", e)))?;

        self.stream = Some(stream);
        log::info!(
            "Audio playback started on device: {}",
            self.device_info.name
        );
        Ok(())
    }

    /// Stop playing audio
    pub fn stop(&mut self) {
        if self.stream.take().is_some() {
            log::info!(
                "Audio playback stopped on device: {}",
                self.device_info.name
            );
        }
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl Drop for AudioOutput {
    fn drop(&mut self) {
        self.stop();
    }
}
