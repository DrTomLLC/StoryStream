//! Integration tests for the configuration system

use std::path::PathBuf;
use storystream_config::{
    AppConfig, Config, ConfigManager, ConfigSection, LibraryConfig, PlayerConfig, CONFIG_VERSION,
};
use tempfile::TempDir;

fn setup_test_manager() -> Result<(TempDir, ConfigManager), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = ConfigManager::with_directory(temp_dir.path().to_path_buf())?;
    Ok((temp_dir, manager))
}

#[test]
fn test_full_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;

    let created = manager.initialize()?;
    assert!(created);

    let config = manager.load()?;
    assert_eq!(config.version, CONFIG_VERSION);

    let mut modified = config.clone();
    modified.player.default_volume = 80;
    modified.library.auto_import = true;
    manager.save(&modified)?;

    let reloaded = manager.load()?;
    assert_eq!(reloaded.player.default_volume, 80);
    assert!(reloaded.library.auto_import);

    manager.reset()?;
    let after_reset = manager.load()?;
    assert_eq!(after_reset, Config::default());

    Ok(())
}

#[test]
fn test_config_validation_integration() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;

    manager.save(&Config::default())?;

    let errors = manager.validate()?;
    assert!(errors.is_empty());

    let mut invalid = Config::default();
    invalid.player.default_volume = 150;
    let result = manager.save(&invalid);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_atomic_save() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;

    let config = Config::default();
    manager.save(&config)?;

    assert!(manager.config_path().exists());

    manager.save(&config)?;

    let backup_path = manager.config_path().with_extension("toml.backup");
    assert!(backup_path.exists());

    Ok(())
}

#[test]
fn test_merge_functionality() {
    let mut base = Config::default();
    let mut override_config = Config::default();

    override_config.player.default_volume = 95;
    override_config.app.debug_mode = true;
    override_config.library.auto_import = true;

    base.merge(override_config);

    assert_eq!(base.player.default_volume, 95);
    assert!(base.app.debug_mode);
    assert!(base.library.auto_import);
}

#[test]
fn test_all_sections_default_are_valid() {
    let app_config = AppConfig::default();
    assert!(app_config.validate().is_ok());

    let player_config = PlayerConfig::default();
    assert!(player_config.validate().is_ok());

    let library_config = LibraryConfig::default();
    assert!(library_config.validate().is_ok());

    let full_config = Config::default();
    assert!(full_config.validate().is_ok());
}

#[test]
fn test_update_closure() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;

    manager.initialize()?;

    manager.update(|config| {
        config.player.default_volume = 65;
        config.player.auto_resume = false;
        config.app.debug_mode = true;
    })?;

    let config = manager.load()?;
    assert_eq!(config.player.default_volume, 65);
    assert!(!config.player.auto_resume);
    assert!(config.app.debug_mode);

    Ok(())
}

#[test]
fn test_graceful_degradation_on_load_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let manager = ConfigManager::with_directory(temp_dir.path().to_path_buf())
        .expect("Failed to create manager");

    let config = manager.load_or_default();
    assert_eq!(config, Config::default());
}

#[test]
fn test_serialization_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let original = Config::default();
    let toml_string = toml::to_string(&original)?;
    let deserialized: Config = toml::from_str(&toml_string)?;
    assert_eq!(original, deserialized);
    Ok(())
}

#[test]
fn test_env_overrides() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;
    manager.initialize()?;

    std::env::set_var("STORYSTREAM_PLAYER_DEFAULT_VOLUME", "88");
    std::env::set_var("STORYSTREAM_PLAYER_DEFAULT_SPEED", "1.5");

    let config = manager.load_with_env_overrides()?;

    assert_eq!(config.player.default_volume, 88);
    assert_eq!(config.player.default_speed, 1.5);

    std::env::remove_var("STORYSTREAM_PLAYER_DEFAULT_VOLUME");
    std::env::remove_var("STORYSTREAM_PLAYER_DEFAULT_SPEED");

    Ok(())
}

#[test]
fn test_library_paths_empty_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    assert!(config.library.library_paths.is_empty());

    let (_temp_dir, manager) = setup_test_manager()?;
    manager.save(&config)?;

    let loaded = manager.load()?;
    assert!(loaded.library.library_paths.is_empty());

    Ok(())
}

#[test]
fn test_library_paths_with_values() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;

    let mut config = Config::default();
    config.library.library_paths = vec![
        PathBuf::from("/home/user/audiobooks"),
        PathBuf::from("/media/audiobooks"),
    ];

    manager.save(&config)?;
    let loaded = manager.load()?;

    assert_eq!(loaded.library.library_paths.len(), 2);
    assert_eq!(
        loaded.library.library_paths[0],
        PathBuf::from("/home/user/audiobooks")
    );

    Ok(())
}

#[test]
fn test_config_version_is_preserved() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;

    let config = Config::default();
    manager.save(&config)?;

    let loaded = manager.load()?;
    assert_eq!(loaded.version, CONFIG_VERSION);

    Ok(())
}

#[test]
fn test_multiple_validation_errors_collected() {
    let mut config = Config::default();
    config.player.default_volume = 150;
    config.player.default_speed = 5.0;
    config.app.max_recent_books = 0;

    let result = config.validate();
    assert!(result.is_err());

    if let Err(errors) = result {
        assert_eq!(errors.len(), 3);
    }
}

#[test]
fn test_supported_audio_extensions() {
    let config = Config::default();

    let expected_extensions = vec!["mp3", "m4a", "m4b", "ogg", "opus", "flac", "wav"];

    for ext in expected_extensions {
        assert!(config
            .library
            .supported_extensions
            .contains(&ext.to_string()));
    }
}
