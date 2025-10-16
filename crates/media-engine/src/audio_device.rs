// crates/media-engine/src/audio_device.rs
// Complete audio device management system

use crate::error::{EngineError, EngineResult};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, SampleRate, SupportedStreamConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about an audio device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    /// Unique identifier for the device
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Whether this is the system default device
    pub is_default: bool,
    /// Supported sample rates
    pub sample_rates: Vec<u32>,
    /// Minimum number of channels
    pub min_channels: u16,
    /// Maximum number of channels
    pub max_channels: u16,
    /// Default/preferred sample rate
    pub default_sample_rate: u32,
    /// Default/preferred channels
    pub default_channels: u16,
}

/// Audio device manager for enumeration and selection
pub struct AudioDeviceManager {
    host: Host,
    devices_cache: HashMap<String, AudioDeviceInfo>,
    selected_device_id: Option<String>,
}

impl AudioDeviceManager {
    /// Create a new audio device manager
    pub fn new() -> EngineResult<Self> {
        let host = cpal::default_host();
        let mut manager = Self {
            host,
            devices_cache: HashMap::new(),
            selected_device_id: None,
        };

        manager.refresh_devices()?;
        Ok(manager)
    }

    /// Refresh the list of available audio devices
    pub fn refresh_devices(&mut self) -> EngineResult<()> {
        self.devices_cache.clear();

        // Get default output device name for comparison
        let default_device = self.host.default_output_device();
        let default_name = default_device
            .as_ref()
            .and_then(|d| d.name().ok());

        // Enumerate all output devices
        let devices = self.host.output_devices()
            .map_err(|e| EngineError::OutputError(format!("Failed to enumerate devices: {}", e)))?;

        for device in devices {
            if let Ok(name) = device.name() {
                let info = self.get_device_info(&device, &name,
                                                default_name.as_ref().map(|n| n == &name).unwrap_or(false))?;
                self.devices_cache.insert(info.id.clone(), info);
            }
        }

        Ok(())
    }

    /// Get information about a specific device
    fn get_device_info(&self, device: &Device, name: &str, is_default: bool) -> EngineResult<AudioDeviceInfo> {
        let configs = device.supported_output_configs()
            .map_err(|e| EngineError::OutputError(format!("Failed to get device configs: {}", e)))?;

        let mut sample_rates = Vec::new();
        let mut min_channels = u16::MAX;
        let mut max_channels = 0;
        let mut default_config: Option<SupportedStreamConfig> = None;

        for config in configs {
            // Collect sample rate range
            let min_sr = config.min_sample_rate().0;
            let max_sr = config.max_sample_rate().0;

            // Add common sample rates within the range
            for &rate in &[8000, 11025, 16000, 22050, 44100, 48000, 88200, 96000, 192000] {
                if rate >= min_sr && rate <= max_sr && !sample_rates.contains(&rate) {
                    sample_rates.push(rate);
                }
            }

            // Track channel range
            let channels = config.channels();
            min_channels = min_channels.min(channels);
            max_channels = max_channels.max(channels);

            // Try to get default config
            if default_config.is_none() {
                default_config = config.try_with_sample_rate(SampleRate(48000))
                    .or_else(|| config.try_with_sample_rate(SampleRate(44100)))
                    .or_else(|| config.try_with_sample_rate(SampleRate(max_sr)));
            }
        }

        sample_rates.sort();

        // Determine defaults
        let (default_sample_rate, default_channels) = if let Some(config) = default_config {
            (config.sample_rate().0, config.channels())
        } else {
            // Fallback defaults
            let default_sr = if sample_rates.contains(&48000) { 48000 }
            else if sample_rates.contains(&44100) { 44100 }
            else { *sample_rates.first().unwrap_or(&44100) };
            (default_sr, 2.min(max_channels).max(min_channels))
        };

        // Generate a stable ID from the device name
        let id = format!("{:x}", md5::compute(name));

        Ok(AudioDeviceInfo {
            id,
            name: name.to_string(),
            is_default,
            sample_rates,
            min_channels,
            max_channels,
            default_sample_rate,
            default_channels,
        })
    }

    /// List all available audio output devices
    pub fn list_devices(&self) -> Vec<AudioDeviceInfo> {
        let mut devices: Vec<_> = self.devices_cache.values().cloned().collect();
        // Sort with default device first, then alphabetically
        devices.sort_by(|a, b| {
            match (a.is_default, b.is_default) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        devices
    }

    /// Get the currently selected device info
    pub fn get_selected_device(&self) -> Option<&AudioDeviceInfo> {
        self.selected_device_id
            .as_ref()
            .and_then(|id| self.devices_cache.get(id))
    }

    /// Select a device by ID
    pub fn select_device(&mut self, device_id: &str) -> EngineResult<()> {
        if !self.devices_cache.contains_key(device_id) {
            return Err(EngineError::OutputError(format!("Device not found: {}", device_id)));
        }
        self.selected_device_id = Some(device_id.to_string());
        Ok(())
    }

    /// Select the default output device
    pub fn select_default_device(&mut self) -> EngineResult<()> {
        let default_device = self.devices_cache
            .values()
            .find(|d| d.is_default)
            .ok_or_else(|| EngineError::OutputError("No default device found".to_string()))?;

        self.selected_device_id = Some(default_device.id.clone());
        Ok(())
    }

    /// Get the actual device for audio output
    pub fn get_output_device(&self) -> EngineResult<Device> {
        // If a specific device is selected, try to use it
        if let Some(device_id) = &self.selected_device_id {
            if let Some(device_info) = self.devices_cache.get(device_id) {
                // Find the device by name
                let devices = self.host.output_devices()
                    .map_err(|e| EngineError::OutputError(format!("Failed to enumerate devices: {}", e)))?;

                for device in devices {
                    if let Ok(name) = device.name() {
                        if name == device_info.name {
                            return Ok(device);
                        }
                    }
                }
            }
        }

        // Fallback to default device
        self.host
            .default_output_device()
            .ok_or_else(|| EngineError::OutputError("No output device available".to_string()))
    }

    /// Check if a device is still available
    pub fn is_device_available(&self, device_id: &str) -> bool {
        if let Some(device_info) = self.devices_cache.get(device_id) {
            // Try to find the device
            if let Ok(devices) = self.host.output_devices() {
                for device in devices {
                    if let Ok(name) = device.name() {
                        if name == device_info.name {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Get recommended settings for the selected device
    pub fn get_recommended_settings(&self) -> Option<(u32, u16)> {
        self.get_selected_device()
            .or_else(|| self.devices_cache.values().find(|d| d.is_default))
            .map(|info| (info.default_sample_rate, info.default_channels))
    }
}

impl Default for AudioDeviceManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            host: cpal::default_host(),
            devices_cache: HashMap::new(),
            selected_device_id: None,
        })
    }
}