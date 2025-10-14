//! Property-based tests for configuration system

use storystream_config::{Config, ConfigManager};
use tempfile::TempDir;

#[test]
fn property_serialization_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let toml_string = toml::to_string(&config)?;
    let deserialized: Config = toml::from_str(&toml_string)?;
    assert_eq!(config, deserialized);
    Ok(())
}

#[test]
fn property_default_always_valid() {
    let config = Config::default();
    assert!(config.validate().is_ok());
}

#[test]
fn property_load_save_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = ConfigManager::with_directory(temp_dir.path().to_path_buf())?;

    let config = Config::default();
    manager.save(&config)?;
    let loaded = manager.load()?;
    manager.save(&loaded)?;
    let loaded2 = manager.load()?;
    assert_eq!(loaded, loaded2);
    Ok(())
}

#[test]
fn property_validation_deterministic() {
    let mut config = Config::default();
    config.player.default_volume = 150;

    let result1 = config.validate();
    let result2 = config.validate();

    assert_eq!(result1.is_err(), result2.is_err());

    if let (Err(e1), Err(e2)) = (result1, result2) {
        assert_eq!(e1.len(), e2.len());
    }
}

#[test]
fn property_merge_preserves_validity() {
    let mut base = Config::default();
    let override_cfg = Config::default();

    assert!(base.validate().is_ok());
    assert!(override_cfg.validate().is_ok());

    base.merge(override_cfg);

    assert!(base.validate().is_ok());
}

#[test]
fn property_all_valid_volumes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = ConfigManager::with_directory(temp_dir.path().to_path_buf())?;

    for volume in 0..=100u8 {
        let mut config = Config::default();
        config.player.default_volume = volume;

        assert!(config.validate().is_ok());
        manager.save(&config)?;

        let loaded = manager.load()?;
        assert_eq!(loaded.player.default_volume, volume);
    }

    Ok(())
}

#[test]
fn property_invalid_configs_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = ConfigManager::with_directory(temp_dir.path().to_path_buf())?;

    let invalid_volumes = [101, 150, 200, 255];

    for &volume in &invalid_volumes {
        let mut config = Config::default();
        config.player.default_volume = volume;

        assert!(config.validate().is_err());
        assert!(manager.save(&config).is_err());
    }

    Ok(())
}

#[test]
fn property_config_file_size_bounded() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = ConfigManager::with_directory(temp_dir.path().to_path_buf())?;

    let mut config = Config::default();

    for i in 0..10 {
        config.library.library_paths.push(std::path::PathBuf::from(format!("/path/{}", i)));
    }

    manager.save(&config)?;

    let metadata = std::fs::metadata(manager.config_path())?;

    assert!(metadata.len() < 10_000);

    Ok(())
}

#[test]
fn property_concurrent_reads_safe() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_dir = temp_dir.path().to_path_buf();
    let manager = ConfigManager::with_directory(config_dir.clone())?;

    manager.initialize()?;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let dir = config_dir.clone();
            std::thread::spawn(move || {
                if let Ok(mgr) = ConfigManager::with_directory(dir) {
                    for _ in 0..10 {
                        let _ = mgr.load();
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        assert!(handle.join().is_ok());
    }

    Ok(())
}

#[test]
fn property_update_never_corrupts() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = ConfigManager::with_directory(temp_dir.path().to_path_buf())?;

    manager.initialize()?;

    for i in 0..20 {
        let volume = (i * 5) % 101;
        let result = manager.update(|config| {
            config.player.default_volume = volume as u8;
        });

        if result.is_ok() {
            assert!(manager.load().is_ok());
        }
    }

    Ok(())
}