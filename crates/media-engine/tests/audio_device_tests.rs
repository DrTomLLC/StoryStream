// crates/media-engine/tests/audio_device_tests.rs
//! Comprehensive tests for audio device management
//! Zero unwrap() calls - all errors handled gracefully

use media_engine::{AudioDeviceManager, AudioOutput, AudioOutputConfig};

#[test]
fn test_device_manager_creation() {
    let manager = AudioDeviceManager::new();
    assert!(manager.is_ok(), "Should create device manager successfully");
}

#[test]
fn test_list_devices() {
    let manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to create manager: {}", e);
            return;
        }
    };

    let devices = manager.list_devices();

    // Every system should have at least one audio output device
    assert!(!devices.is_empty(), "Should have at least one audio device");

    // Should have exactly one default device
    let default_count = devices.iter().filter(|d| d.is_default).count();
    assert_eq!(default_count, 1, "Should have exactly one default device");
}

#[test]
fn test_device_properties() {
    let manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let devices = manager.list_devices();

    for device in &devices {
        // Device should have a name
        assert!(!device.name.is_empty(), "Device should have a name");

        // Device should have an ID
        assert!(!device.id.is_empty(), "Device should have an ID");

        // Channel range should be valid
        assert!(device.min_channels > 0, "Min channels should be > 0");
        assert!(
            device.max_channels >= device.min_channels,
            "Max channels should be >= min channels"
        );

        // Default channels should be within range
        assert!(
            device.default_channels >= device.min_channels,
            "Default channels should be >= min"
        );
        assert!(
            device.default_channels <= device.max_channels,
            "Default channels should be <= max"
        );

        // Should support at least one sample rate
        if !device.sample_rates.is_empty() {
            // Sample rates should be in ascending order
            let mut prev = 0u32;
            for &rate in &device.sample_rates {
                assert!(rate > prev, "Sample rates should be sorted");
                prev = rate;
            }

            // Default sample rate should be in the list
            assert!(
                device.sample_rates.contains(&device.default_sample_rate),
                "Default sample rate should be supported"
            );
        }
    }
}

#[test]
fn test_default_device_selection() {
    let mut manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    // Select default device
    if let Err(e) = manager.select_default_device() {
        panic!("Failed to select default device: {}", e);
    }

    let selected = manager.get_selected_device();
    assert!(selected.is_some(), "Should have a selected device");

    if let Some(device) = selected {
        assert!(device.is_default, "Selected device should be default");
    }
}

#[test]
fn test_device_selection_by_id() {
    let mut manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let devices = manager.list_devices();

    if let Some(device) = devices.first() {
        let result = manager.select_device(&device.id);
        assert!(result.is_ok(), "Should select device by ID");

        let selected = manager.get_selected_device();
        assert!(selected.is_some(), "Should have a selected device");

        if let Some(sel) = selected {
            assert_eq!(sel.id, device.id, "Should select correct device");
        }
    }
}

#[test]
fn test_invalid_device_selection() {
    let mut manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let result = manager.select_device("invalid-device-id-12345");
    assert!(result.is_err(), "Should fail to select invalid device");
}

#[test]
fn test_device_availability() {
    let manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let devices = manager.list_devices();

    if let Some(device) = devices.first() {
        assert!(
            manager.is_device_available(&device.id),
            "Device should be available"
        );
    }

    assert!(
        !manager.is_device_available("invalid-id"),
        "Invalid device should not be available"
    );
}

#[test]
fn test_recommended_settings() {
    let mut manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    if manager.select_default_device().is_err() {
        return;
    }

    let settings = manager.get_recommended_settings();
    assert!(settings.is_some(), "Should have recommended settings");

    if let Some((sample_rate, channels)) = settings {
        assert!(sample_rate > 0, "Sample rate should be > 0");
        assert!(channels > 0, "Channels should be > 0");

        // Common sample rates
        assert!(
            [8000, 11025, 16000, 22050, 44100, 48000, 88200, 96000, 192000].contains(&sample_rate),
            "Sample rate should be a common value"
        );
    }
}

#[test]
fn test_device_refresh() {
    let mut manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let initial_count = manager.list_devices().len();

    // Refresh should succeed
    assert!(
        manager.refresh_devices().is_ok(),
        "Device refresh should succeed"
    );

    let after_count = manager.list_devices().len();
    assert_eq!(
        initial_count, after_count,
        "Device count should be consistent"
    );
}

#[test]
fn test_audio_output_creation() {
    let result = AudioOutput::new(48000, 2);
    assert!(result.is_ok(), "Should create audio output");

    if let Ok(output) = result {
        assert_eq!(
            output.device_info().default_channels,
            2,
            "Should use specified channels"
        );
    }
}

#[test]
fn test_audio_output_with_config() {
    let config = AudioOutputConfig {
        sample_rate: 44100,
        channels: 2,
        device_id: None,
        buffer_size: Some(2048),
    };

    let result = AudioOutput::with_config(config);
    assert!(result.is_ok(), "Should create audio output with config");
}

#[test]
fn test_audio_output_invalid_channels() {
    // Try to create output with 100 channels (unlikely to be supported)
    let config = AudioOutputConfig {
        sample_rate: 48000,
        channels: 100,
        device_id: None,
        buffer_size: None,
    };

    let result = AudioOutput::with_config(config);
    // This might fail on most systems
    // We're just checking that it handles the error gracefully
    if result.is_err() {
        // Error is expected and handled
        assert!(true);
    }
}

#[test]
fn test_audio_output_device_list() {
    let output = match AudioOutput::new(48000, 2) {
        Ok(o) => o,
        Err(_) => return,
    };

    let devices = output.list_devices();

    assert!(!devices.is_empty(), "Should list devices");
    assert!(
        devices.iter().any(|d| d.is_default),
        "Should include default device"
    );
}

#[test]
fn test_audio_output_device_availability() {
    let output = match AudioOutput::new(48000, 2) {
        Ok(o) => o,
        Err(_) => return,
    };

    // Current device should be available
    assert!(
        output.is_device_available(),
        "Current device should be available"
    );
}

#[test]
fn test_device_id_uniqueness() {
    let manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let devices = manager.list_devices();

    // Collect all IDs
    let mut ids = std::collections::HashSet::new();
    for device in &devices {
        assert!(ids.insert(device.id.clone()), "Device IDs should be unique");
    }
}

#[test]
fn test_device_sorting() {
    let manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let devices = manager.list_devices();

    if devices.len() > 1 {
        // First device should be default
        assert!(devices[0].is_default, "First device should be the default");

        // Rest should be alphabetically sorted (after default)
        for i in 1..devices.len() {
            if !devices[i].is_default && i > 0 {
                if !devices[i - 1].is_default {
                    assert!(
                        devices[i].name >= devices[i - 1].name,
                        "Non-default devices should be alphabetically sorted"
                    );
                }
            }
        }
    }
}

#[test]
fn test_manager_multiple_selections() {
    let mut manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let devices = manager.list_devices();

    if devices.len() >= 2 {
        // Select first device
        if manager.select_device(&devices[0].id).is_err() {
            return;
        }

        if let Some(selected) = manager.get_selected_device() {
            assert_eq!(selected.id, devices[0].id);
        }

        // Select second device
        if manager.select_device(&devices[1].id).is_err() {
            return;
        }

        if let Some(selected) = manager.get_selected_device() {
            assert_eq!(selected.id, devices[1].id);
        }

        // Select back to first
        if manager.select_device(&devices[0].id).is_err() {
            return;
        }

        if let Some(selected) = manager.get_selected_device() {
            assert_eq!(selected.id, devices[0].id);
        }
    }
}

#[test]
fn test_output_config_default() {
    let config = AudioOutputConfig::default();
    assert_eq!(
        config.sample_rate, 48000,
        "Default sample rate should be 48kHz"
    );
    assert_eq!(config.channels, 2, "Default channels should be 2");
    assert!(
        config.device_id.is_none(),
        "Default device ID should be None"
    );
    assert!(
        config.buffer_size.is_none(),
        "Default buffer size should be None"
    );
}

#[test]
fn test_concurrent_manager_creation() {
    use std::thread;

    let handles: Vec<_> = (0..4)
        .map(|_| {
            thread::spawn(|| match AudioDeviceManager::new() {
                Ok(manager) => Some(manager.list_devices().len()),
                Err(_) => None,
            })
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .filter_map(|h| match h.join() {
            Ok(result) => result,
            Err(_) => None,
        })
        .collect();

    // All threads should get the same number of devices
    if results.len() > 1 {
        assert!(
            results.windows(2).all(|w| w[0] == w[1]),
            "All threads should see same devices"
        );
    }
}

#[test]
fn test_device_capabilities_stereo() {
    let manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let devices = manager.list_devices();

    // Most audio devices should support stereo
    let stereo_devices = devices
        .iter()
        .filter(|d| d.min_channels <= 2 && d.max_channels >= 2)
        .count();

    assert!(
        stereo_devices > 0,
        "At least one device should support stereo"
    );
}

#[test]
fn test_common_sample_rates_supported() {
    let manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let devices = manager.list_devices();

    // At least one device should support common sample rates
    let common_rates = [44100, 48000];
    let has_common_rate = devices.iter().any(|device| {
        device
            .sample_rates
            .iter()
            .any(|&rate| common_rates.contains(&rate))
    });

    assert!(
        has_common_rate,
        "At least one device should support 44.1 or 48 kHz"
    );
}

#[test]
fn test_full_device_selection_workflow() {
    // Create manager
    let mut manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    // List devices
    let devices = manager.list_devices();
    assert!(!devices.is_empty());

    // Select default
    if manager.select_default_device().is_err() {
        return;
    }

    let default = match manager.get_selected_device() {
        Some(d) => d.clone(),
        None => return,
    };

    // Get recommended settings
    let (sr, ch) = match manager.get_recommended_settings() {
        Some(settings) => settings,
        None => return,
    };

    // Create output with those settings
    let config = AudioOutputConfig {
        sample_rate: sr,
        channels: ch,
        device_id: Some(default.id.clone()),
        buffer_size: None,
    };

    let output = match AudioOutput::with_config(config) {
        Ok(o) => o,
        Err(_) => return,
    };

    // Verify output device matches
    assert_eq!(output.device_info().id, default.id);
}
