// examples/list_audio_devices.rs
use media_engine::AudioDeviceManager;

fn main() {
    let manager = AudioDeviceManager::new().unwrap();

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