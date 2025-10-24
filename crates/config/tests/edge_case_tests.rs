//! Edge case and error scenario tests

use std::fs;
use std::path::PathBuf;
use storystream_config::{Config, ConfigManager};
use tempfile::TempDir;

fn setup_test_manager() -> Result<(TempDir, ConfigManager), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = ConfigManager::with_directory(temp_dir.path().to_path_buf())?;
    Ok((temp_dir, manager))
}

#[test]
fn test_corrupted_config_uses_defaults() {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir, manager) = setup_test_manager()?;

        let config_path = manager.config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&config_path, "this is not valid TOML {{{")?;

        let config = manager.load_or_default();
        assert_eq!(config, Config::default());

        Ok(())
    })();

    assert!(result.is_ok());
}

#[test]
fn test_save_creates_parent_directories() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let nested_path = temp_dir.path().join("a").join("b").join("c");
    let manager = ConfigManager::with_directory(nested_path)?;

    let config = Config::default();
    manager.save(&config)?;

    assert!(manager.config_path().exists());

    Ok(())
}

#[test]
fn test_concurrent_config_loads() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_dir = temp_dir.path().to_path_buf();
    let manager = ConfigManager::with_directory(config_dir.clone())?;
    manager.initialize()?;

    let config_dir1 = config_dir.clone();
    let config_dir2 = config_dir.clone();

    let handle1 = std::thread::spawn(move || {
        if let Ok(mgr) = ConfigManager::with_directory(config_dir1) {
            for _ in 0..10 {
                let _ = mgr.load();
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    });

    let handle2 = std::thread::spawn(move || {
        if let Ok(mgr) = ConfigManager::with_directory(config_dir2) {
            for _ in 0..10 {
                let _ = mgr.load();
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    });

    assert!(handle1.join().is_ok());
    assert!(handle2.join().is_ok());

    Ok(())
}

#[test]
fn test_boundary_values_validation() {
    let mut config = Config::default();

    config.player.default_volume = 100;
    config.player.default_speed = 2.0;
    assert!(config.validate().is_ok());

    config.player.default_volume = 0;
    config.player.default_speed = 0.5;
    assert!(config.validate().is_ok());

    config.player.default_volume = 101;
    assert!(config.validate().is_err());

    config.player.default_volume = 100;
    config.player.default_speed = 2.1;
    assert!(config.validate().is_err());
}

#[test]
fn test_empty_library_paths() {
    let config = Config::default();
    assert!(config.library.library_paths.is_empty());
    assert!(config.validate().is_ok());
}

#[test]
fn test_special_characters_in_paths() {
    let mut config = Config::default();

    config.library.library_paths = vec![
        PathBuf::from("/path/with spaces/audiobooks"),
        PathBuf::from("/path/with-dashes/audiobooks"),
        PathBuf::from("/path/with_underscores/audiobooks"),
        PathBuf::from("/path/with.dots/audiobooks"),
    ];

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_very_long_paths() {
    let mut config = Config::default();

    let long_segment = "very_long_directory_name_that_keeps_going";
    let mut long_path = PathBuf::from("/");
    for _ in 0..20 {
        long_path.push(long_segment);
    }

    config.library.library_paths = vec![long_path];

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_unicode_in_extensions() {
    let mut config = Config::default();

    config.library.supported_extensions = vec![
        "mp3".to_string(),
        "файл".to_string(),
        "文件".to_string(),
        "ファイル".to_string(),
    ];

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_whitespace_in_extensions() {
    let mut config = Config::default();

    config.library.supported_extensions = vec![" mp3 ".to_string(), "  m4a  ".to_string()];

    config.library.supported_extensions.push("   ".to_string());
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_minimum_autosave_interval() {
    let mut config = Config::default();

    config.player.autosave_interval_secs = 1;
    assert!(config.validate().is_ok());

    config.player.autosave_interval_secs = 0;
    assert!(config.validate().is_err());
}

#[test]
#[cfg(unix)]
fn test_readonly_config_file() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let (_temp_dir, manager) = setup_test_manager()?;
    manager.initialize()?;

    let config_path = manager.config_path();

    let mut perms = fs::metadata(&config_path)?.permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&config_path, perms)?;

    let config = Config::default();
    let result = manager.save(&config);
    assert!(result.is_err());

    let loaded = manager.load();
    assert!(loaded.is_ok());

    Ok(())
}

#[test]
fn test_rapid_saves() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;
    let mut config = Config::default();

    for i in 0..100 {
        config.player.default_volume = (i % 100) as u8;
        manager.save(&config)?;
    }

    Ok(())
}

#[test]
fn test_merge_with_defaults() {
    let mut base = Config::default();
    let defaults = Config::default();

    base.player.default_volume = 90;
    base.merge(defaults);

    assert_eq!(base.player.default_volume, 70);
}

#[test]
fn test_all_validation_errors_collected() {
    let mut config = Config::default();

    config.player.default_volume = 150;
    config.player.default_speed = 5.0;
    config.player.autosave_interval_secs = 0;
    config.app.max_recent_books = 0;

    let result = config.validate();
    assert!(result.is_err());

    if let Err(errors) = result {
        assert!(errors.len() >= 4);
    }
}

#[test]
fn test_organize_files_without_target() {
    let mut config = Config::default();

    config.library.organize_files = true;
    config.library.organization_target = None;

    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_config_file_deleted_during_operation() {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir, manager) = setup_test_manager()?;
        manager.initialize()?;

        let config_path = manager.config_path();
        fs::remove_file(&config_path)?;

        let config = manager.load_or_default();
        assert_eq!(config, Config::default());

        Ok(())
    })();

    assert!(result.is_ok());
}

#[test]
fn test_empty_config_file() {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir, manager) = setup_test_manager()?;

        let config_path = manager.config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&config_path, "")?;

        let result = manager.load();
        assert!(result.is_err());

        let config = manager.load_or_default();
        assert_eq!(config, Config::default());

        Ok(())
    })();

    assert!(result.is_ok());
}

#[test]
fn test_partial_config_toml() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;

    let partial_toml = r#"
version = 1

[player]
default_volume = 80
"#;

    let config_path = manager.config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&config_path, partial_toml)?;

    let config = manager.load()?;
    assert_eq!(config.player.default_volume, 80);
    assert_eq!(config.player.default_speed, 1.0);

    Ok(())
}

#[test]
fn test_update_with_invalid_value() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;
    manager.initialize()?;

    let result = manager.update(|config| {
        config.player.default_volume = 200;
    });

    assert!(result.is_err());

    let config = manager.load()?;
    assert_eq!(config.player.default_volume, 70);

    Ok(())
}

#[test]
fn test_many_library_paths() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::default();

    for i in 0..1000 {
        config
            .library
            .library_paths
            .push(PathBuf::from(format!("/path/to/audiobooks/{}", i)));
    }

    assert!(config.validate().is_ok());

    let toml = toml::to_string(&config)?;
    let deserialized: Config = toml::from_str(&toml)?;
    assert_eq!(deserialized.library.library_paths.len(), 1000);

    Ok(())
}

#[test]
fn test_playback_speed_precision() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::default();

    config.player.default_speed = 1.23456789;

    let toml = toml::to_string(&config)?;
    let deserialized: Config = toml::from_str(&toml)?;

    assert!((deserialized.player.default_speed - 1.23456789).abs() < 0.0001);

    Ok(())
}

#[test]
fn test_backup_preserved_on_failed_save() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, manager) = setup_test_manager()?;

    let mut config = Config::default();
    config.player.default_volume = 75;
    manager.save(&config)?;

    config.player.default_volume = 200;
    let result = manager.save(&config);

    assert!(result.is_err());

    let backup_path = manager.config_path().with_extension("toml.backup");
    if backup_path.exists() {
        let backup_contents = fs::read_to_string(&backup_path)?;
        let backup_config: Config = toml::from_str(&backup_contents)?;
        assert_eq!(backup_config.player.default_volume, 75);
    }

    Ok(())
}
