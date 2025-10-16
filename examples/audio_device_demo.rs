// crates/media-engine/examples/audio_device_demo.rs
use media_engine::{AudioDeviceManager, AudioOutput, AudioOutputConfig};
use std::io::{self, Write};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    println!("StoryStream Audio Device Manager Demo");
    println!("=====================================\n");

    let mut manager = AudioDeviceManager::new()
        .map_err(|e| format!("Failed to create device manager: {}", e))?;

    list_all_devices(&manager);
    show_default_device(&mut manager)?;

    if confirm("Would you like to select a different device? (y/n): ")? {
        if let Err(e) = select_device_interactive(&mut manager) {
            eprintln!("Warning: {}", e);
        }
    }

    if confirm("\nWould you like to test audio output creation? (y/n): ")? {
        if let Err(e) = test_audio_output(&manager) {
            eprintln!("Warning: {}", e);
        }
    }

    println!("\nDemo complete!");
    Ok(())
}

fn list_all_devices(manager: &AudioDeviceManager) {
    println!("Available Audio Output Devices:");
    println!("-------------------------------");

    let devices = manager.list_devices();

    for (idx, device) in devices.iter().enumerate() {
        println!("\n{}. {} {}",
                 idx + 1,
                 if device.is_default { "[DEFAULT]" } else { "         " },
                 device.name
        );
        println!("   ID: {}", device.id);
        println!("   Channels: {} to {} (default: {})",
                 device.min_channels,
                 device.max_channels,
                 device.default_channels
        );

        if !device.sample_rates.is_empty() {
            println!("   Sample Rates:");
            for rate in &device.sample_rates {
                print!("      {} Hz", rate);
                if *rate == device.default_sample_rate {
                    print!(" [DEFAULT]");
                }
                println!();
            }
        } else {
            println!("   Sample Rates: Supports all rates");
        }
    }
    println!();
}

fn show_default_device(manager: &mut AudioDeviceManager) -> Result<(), String> {
    manager.select_default_device()
        .map_err(|e| format!("Failed to select default device: {}", e))?;

    if let Some(device) = manager.get_selected_device() {
        println!("\nCurrent Default Device:");
        println!("----------------------");
        println!("Name: {}", device.name);
        println!("ID: {}", device.id);

        if let Some((sr, ch)) = manager.get_recommended_settings() {
            println!("\nRecommended Settings:");
            println!("  Sample Rate: {} Hz", sr);
            println!("  Channels: {}", ch);
        }
    }

    Ok(())
}

fn select_device_interactive(manager: &mut AudioDeviceManager) -> Result<(), String> {
    let devices = manager.list_devices();

    println!("\nSelect a device by number (1-{}): ", devices.len());
    print!("> ");
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;

    let choice: usize = input.trim().parse()
        .map_err(|_| "Invalid number".to_string())?;

    if choice == 0 || choice > devices.len() {
        return Err("Choice out of range".to_string());
    }

    let device = &devices[choice - 1];
    manager.select_device(&device.id)
        .map_err(|e| format!("Failed to select device: {}", e))?;

    println!("\n✓ Selected: {}", device.name);

    if !manager.is_device_available(&device.id) {
        println!("⚠ Warning: Device may not be available");
    }

    Ok(())
}

fn test_audio_output(manager: &AudioDeviceManager) -> Result<(), String> {
    let selected = manager.get_selected_device()
        .ok_or("No device selected")?;

    println!("\nCreating audio output for: {}", selected.name);

    println!("  Testing with 48kHz stereo...");
    match AudioOutput::new(48000, 2) {
        Ok(output) => {
            println!("  ✓ Created successfully");
            println!("    Device: {}", output.device_info().name);
            println!("    Available: {}", output.is_device_available());
        }
        Err(e) => {
            println!("  ✗ Failed: {}", e);
        }
    }

    println!("\n  Testing with custom configuration...");
    let config = AudioOutputConfig {
        sample_rate: selected.default_sample_rate,
        channels: selected.default_channels,
        device_id: Some(selected.id.clone()),
        buffer_size: Some(2048),
    };

    match AudioOutput::with_config(config) {
        Ok(_) => {
            println!("  ✓ Created with custom config");
            println!("    Sample Rate: {} Hz", selected.default_sample_rate);
            println!("    Channels: {}", selected.default_channels);
            println!("    Buffer Size: 2048 samples");
        }
        Err(e) => {
            println!("  ✗ Failed: {}", e);
        }
    }

    Ok(())
}

fn confirm(prompt: &str) -> Result<bool, String> {
    print!("{}", prompt);
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}