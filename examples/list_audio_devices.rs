// crates/media-engine/examples/list_audio_devices.rs
use media_engine::AudioDeviceManager;

fn main() {
    let manager = match AudioDeviceManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to initialize audio device manager: {}", e);
            std::process::exit(1);
        }
    };

    println!("Available Audio Devices:");
    println!("========================");

    for device in manager.list_devices() {
        println!("\n{} {}",
                 if device.is_default { "📢" } else { "  " },
                 device.name
        );
        println!("  ID: {}", device.id);
        println!("  Channels: {}-{} (default: {})",
                 device.min_channels,
                 device.max_channels,
                 device.default_channels
        );
        println!("  Sample rates: {:?}", device.sample_rates);
    }
}