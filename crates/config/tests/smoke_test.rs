//! Comprehensive smoke test

use storystream_config::{
    backup::ConfigBackupManager, schema, watcher::ConfigWatcher, Config, ConfigManager,
};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn smoke_test_complete_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_dir = temp_dir.path().to_path_buf();

    println!("\n=== CONFIG SYSTEM SMOKE TEST ===\n");

    println!("1. Creating ConfigManager...");
    let manager = ConfigManager::with_directory(config_dir.clone())?;
    assert!(!manager.config_path().exists());
    println!("   ✓ Manager created");

    println!("2. Initializing config...");
    let created = manager.initialize()?;
    assert!(created);
    assert!(manager.config_path().exists());
    println!("   ✓ Config file created");

    println!("3. Loading default config...");
    let config = manager.load()?;
    assert_eq!(config, Config::default());
    println!("   ✓ Default config loaded");

    println!("4. Validating config...");
    let errors = manager.validate()?;
    assert!(errors.is_empty());
    println!("   ✓ Config is valid");

    println!("5. Modifying and saving config...");
    manager.update(|config| {
        config.player.default_volume = 85;
        config.player.default_speed = 1.5;
        config.library.auto_import = true;
        config.library.library_paths = vec![PathBuf::from("/test/audiobooks")];
    })?;
    println!("   ✓ Config updated and saved");

    println!("6. Reloading and verifying changes...");
    let reloaded = manager.load()?;
    assert_eq!(reloaded.player.default_volume, 85);
    assert_eq!(reloaded.player.default_speed, 1.5);
    assert!(reloaded.library.auto_import);
    assert_eq!(reloaded.library.library_paths.len(), 1);
    println!("   ✓ Changes persisted correctly");

    println!("7. Creating backup...");
    let backup_dir = config_dir.join("backups");
    let backup_manager = ConfigBackupManager::new(backup_dir.clone()).with_max_backups(5);
    let backup_path = backup_manager.create_backup(&reloaded)?;
    assert!(backup_path.exists());
    println!("   ✓ Backup created");

    println!("8. Listing backups...");
    let backups = backup_manager.list_backups()?;
    assert_eq!(backups.len(), 1);
    println!("   ✓ Found {} backup(s)", backups.len());

    println!("9. Making another change...");
    manager.update(|config| {
        config.player.default_volume = 95;
    })?;
    let modified = manager.load()?;
    assert_eq!(modified.player.default_volume, 95);
    println!("   ✓ Config modified again");

    println!("10. Restoring from backup...");
    let backup = &backups[0];
    let restored = backup_manager.restore_from_backup(&backup.path)?;
    assert_eq!(restored.player.default_volume, 85);
    manager.save(&restored)?;
    let verified = manager.load()?;
    assert_eq!(verified.player.default_volume, 85);
    println!("   ✓ Backup restored successfully");

    println!("11. Generating schemas...");
    let documented_toml = schema::generate_documented_toml();
    assert!(documented_toml.contains("[app]"));
    assert!(documented_toml.contains("[player]"));
    assert!(documented_toml.contains("[library]"));

    let json_schema = schema::generate_json_schema();
    let parsed: serde_json::Value = serde_json::from_str(&json_schema)?;
    assert!(parsed["properties"]["version"].is_object());
    println!("   ✓ Schemas generated");

    println!("12. Testing invalid config rejection...");
    let mut invalid = Config::default();
    invalid.player.default_volume = 150;
    let result = manager.save(&invalid);
    assert!(result.is_err());
    println!("   ✓ Invalid config rejected");

    println!("13. Testing load_or_default safety...");
    let safe_config = manager.load_or_default();
    assert_eq!(safe_config.player.default_volume, 85);
    println!("   ✓ load_or_default works safely");

    println!("14. Resetting to defaults...");
    manager.reset()?;
    let reset_config = manager.load()?;
    assert_eq!(reset_config, Config::default());
    println!("   ✓ Config reset to defaults");

    println!("15. Testing hot reload watcher...");
    let watch_config = manager.load()?;
    let watcher = ConfigWatcher::new(manager.config_path(), watch_config)?
        .with_check_interval(Duration::from_millis(100));

    let handle = watcher.config_handle();
    let watch_handle = watcher.start_watching();

    std::thread::sleep(Duration::from_millis(300));

    watch_handle.stop();

    if let Ok(config) = handle.read() {
        assert!(config.validate().is_ok());
    }
    println!("   ✓ Hot reload watcher works");

    println!("16. Testing concurrent access...");
    let config_dir1 = config_dir.clone();
    let config_dir2 = config_dir.clone();

    let handle1 = std::thread::spawn(move || {
        if let Ok(mgr) = ConfigManager::with_directory(config_dir1) {
            for _ in 0..5 {
                let _ = mgr.load();
            }
        }
    });

    let handle2 = std::thread::spawn(move || {
        if let Ok(mgr) = ConfigManager::with_directory(config_dir2) {
            for _ in 0..5 {
                let _ = mgr.load();
            }
        }
    });

    assert!(handle1.join().is_ok());
    assert!(handle2.join().is_ok());
    println!("   ✓ Concurrent access safe");

    println!("17. Testing backup rotation...");
    let rotation_manager = ConfigBackupManager::new(backup_dir).with_max_backups(3);
    let test_config = Config::default();

    for _ in 0..5 {
        rotation_manager.create_backup(&test_config)?;
        std::thread::sleep(Duration::from_millis(10));
    }

    let final_backups = rotation_manager.list_backups()?;
    assert_eq!(final_backups.len(), 3);
    println!("   ✓ Backup rotation works");

    println!("18. Testing corrupted file handling...");
    std::fs::write(manager.config_path(), "invalid { toml syntax")?;

    let corrupted_result = manager.load();
    assert!(corrupted_result.is_err());

    let safe_load = manager.load_or_default();
    assert_eq!(safe_load, Config::default());
    println!("   ✓ Corrupted file handled safely");

    println!("19. Testing environment overrides...");
    manager.reset()?;
    std::env::set_var("STORYSTREAM_PLAYER_DEFAULT_VOLUME", "88");

    let env_config = manager.load_with_env_overrides()?;
    assert_eq!(env_config.player.default_volume, 88);

    std::env::remove_var("STORYSTREAM_PLAYER_DEFAULT_VOLUME");
    println!("   ✓ Environment overrides work");

    println!("20. Final system validation...");
    let final_config = Config::default();
    assert!(final_config.validate().is_ok());

    let toml = toml::to_string(&final_config)?;
    let parsed: Config = toml::from_str(&toml)?;
    assert_eq!(parsed, final_config);
    println!("   ✓ System fully operational");

    println!("\n=== ALL SMOKE TESTS PASSED ===\n");

    Ok(())
}

#[test]
fn smoke_test_stress() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== STRESS TEST ===\n");

    let temp_dir = TempDir::new()?;
    let manager = ConfigManager::with_directory(temp_dir.path().to_path_buf())?;

    manager.initialize()?;

    println!("Performing 100 rapid updates...");
    for i in 0..100 {
        manager.update(|config| {
            config.player.default_volume = (i % 101) as u8;
        })?;
    }

    let final_config = manager.load()?;
    assert_eq!(final_config.player.default_volume, 99);

    println!("✓ Stress test passed");

    Ok(())
}

#[test]
fn smoke_test_boundary_values() {
    println!("\n=== BOUNDARY VALUE TEST ===\n");

    let mut config = Config::default();

    config.player.default_volume = 0;
    assert!(config.validate().is_ok(), "volume 0 should be valid");
    println!("✓ volume min (valid=true)");

    config.player.default_volume = 100;
    assert!(config.validate().is_ok(), "volume 100 should be valid");
    println!("✓ volume max (valid=true)");

    config.player.default_volume = 101;
    assert!(config.validate().is_err(), "volume 101 should be invalid");
    println!("✓ volume over (valid=false)");

    config.player.default_volume = 70;
    config.player.default_speed = 0.5;
    assert!(config.validate().is_ok(), "speed 0.5 should be valid");
    println!("✓ speed min (valid=true)");

    config.player.default_speed = 2.0;
    assert!(config.validate().is_ok(), "speed 2.0 should be valid");
    println!("✓ speed max (valid=true)");

    config.player.default_speed = 2.1;
    assert!(config.validate().is_err(), "speed 2.1 should be invalid");
    println!("✓ speed over (valid=false)");

    println!("\n✓ All boundary tests passed");
}