//! Configuration backup and restore utilities
//!
//! This module provides safe backup and restore functionality for configs,
//! including timestamped backups and automatic rotation.

use crate::{Config, ConfigError, ConfigResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Manages configuration backups
pub struct ConfigBackupManager {
    backup_dir: PathBuf,
    max_backups: usize,
}

impl ConfigBackupManager {
    /// Creates a new backup manager
    ///
    /// Backups are stored in the specified directory.
    pub fn new(backup_dir: PathBuf) -> Self {
        Self {
            backup_dir,
            max_backups: 10,
        }
    }

    /// Sets the maximum number of backups to keep
    pub fn with_max_backups(mut self, max: usize) -> Self {
        self.max_backups = max;
        self
    }

    /// Creates a backup of the given config
    ///
    /// Returns the path to the backup file.
    pub fn create_backup(&self, config: &Config) -> ConfigResult<PathBuf> {
        self.ensure_backup_dir()?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S_%3f");
        let backup_filename = format!("config_{}.toml", timestamp);
        let backup_path = self.backup_dir.join(backup_filename);

        let toml_string = toml::to_string_pretty(config)?;
        fs::write(&backup_path, toml_string).map_err(|e| ConfigError::WriteError {
            path: backup_path.clone(),
            source: e,
        })?;

        log::info!("Created config backup at {}", backup_path.display());

        self.rotate_backups()?;

        Ok(backup_path)
    }

    /// Restores config from a backup file
    pub fn restore_from_backup(&self, backup_path: &Path) -> ConfigResult<Config> {
        if !backup_path.exists() {
            return Err(ConfigError::PathResolutionError {
                reason: format!("Backup file not found: {}", backup_path.display()),
            });
        }

        let contents = fs::read_to_string(backup_path).map_err(|e| ConfigError::ReadError {
            path: backup_path.to_path_buf(),
            source: e,
        })?;

        let config: Config = toml::from_str(&contents).map_err(|e| ConfigError::ParseError {
            path: backup_path.to_path_buf(),
            source: e,
        })?;

        if let Err(errors) = config.validate() {
            let error_msg = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            log::warn!("Restored config has validation warnings: {}", error_msg);
        }

        log::info!("Restored config from {}", backup_path.display());

        Ok(config)
    }

    /// Lists all available backups
    pub fn list_backups(&self) -> ConfigResult<Vec<BackupInfo>> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        let entries = fs::read_dir(&self.backup_dir).map_err(ConfigError::IoError)?;

        for entry in entries {
            let entry = entry.map_err(ConfigError::IoError)?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.starts_with("config_") {
                        let metadata = fs::metadata(&path).map_err(ConfigError::IoError)?;
                        let created = metadata
                            .created()
                            .or_else(|_| metadata.modified())
                            .map_err(ConfigError::IoError)?;

                        backups.push(BackupInfo {
                            path: path.clone(),
                            filename: filename.to_string(),
                            created,
                            size_bytes: metadata.len(),
                        });
                    }
                }
            }
        }

        backups.sort_by(|a, b| b.created.cmp(&a.created));

        Ok(backups)
    }

    /// Deletes a specific backup
    pub fn delete_backup(&self, backup_path: &Path) -> ConfigResult<()> {
        if !backup_path.starts_with(&self.backup_dir) {
            return Err(ConfigError::PathResolutionError {
                reason: "Backup path is not in the backup directory".to_string(),
            });
        }

        fs::remove_file(backup_path).map_err(|e| ConfigError::WriteError {
            path: backup_path.to_path_buf(),
            source: e,
        })?;

        log::info!("Deleted backup at {}", backup_path.display());

        Ok(())
    }

    /// Deletes all backups
    pub fn delete_all_backups(&self) -> ConfigResult<usize> {
        let backups = self.list_backups()?;
        let count = backups.len();

        for backup in backups {
            self.delete_backup(&backup.path)?;
        }

        log::info!("Deleted {} backup(s)", count);

        Ok(count)
    }

    /// Ensures the backup directory exists
    fn ensure_backup_dir(&self) -> ConfigResult<()> {
        if !self.backup_dir.exists() {
            fs::create_dir_all(&self.backup_dir).map_err(|e| {
                ConfigError::DirectoryCreationError {
                    path: self.backup_dir.clone(),
                    source: e,
                }
            })?;
            log::info!("Created backup directory: {}", self.backup_dir.display());
        }
        Ok(())
    }

    /// Rotates backups, keeping only the most recent ones
    fn rotate_backups(&self) -> ConfigResult<()> {
        let mut backups = self.list_backups()?;

        if backups.len() > self.max_backups {
            backups.sort_by(|a, b| a.created.cmp(&b.created));

            let to_delete = backups.len() - self.max_backups;
            for backup in backups.iter().take(to_delete) {
                self.delete_backup(&backup.path)?;
            }

            log::info!("Rotated backups, deleted {} old backup(s)", to_delete);
        }

        Ok(())
    }
}

/// Information about a config backup
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// Full path to the backup file
    pub path: PathBuf,
    /// Filename of the backup
    pub filename: String,
    /// Creation timestamp
    pub created: std::time::SystemTime,
    /// File size in bytes
    pub size_bytes: u64,
}

impl BackupInfo {
    /// Returns a human-readable timestamp
    pub fn created_timestamp(&self) -> String {
        use std::time::UNIX_EPOCH;

        self.created
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|d| {
                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Returns file size in a human-readable format
    pub fn size_human(&self) -> String {
        let bytes = self.size_bytes as f64;
        if bytes < 1024.0 {
            format!("{} B", bytes)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.1} KB", bytes / 1024.0)
        } else {
            format!("{:.1} MB", bytes / (1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_backup_manager() -> (TempDir, ConfigBackupManager) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let backup_dir = temp_dir.path().join("backups");
        let manager = ConfigBackupManager::new(backup_dir);
        (temp_dir, manager)
    }

    #[test]
    fn test_create_backup() {
        let (_temp_dir, manager) = setup_backup_manager();
        let config = Config::default();

        let backup_path = manager
            .create_backup(&config)
            .expect("Should create backup");
        assert!(backup_path.exists());
    }

    #[test]
    fn test_restore_backup() {
        let (_temp_dir, manager) = setup_backup_manager();
        let mut config = Config::default();
        config.player.default_volume = 85;

        let backup_path = manager
            .create_backup(&config)
            .expect("Should create backup");

        let restored = manager
            .restore_from_backup(&backup_path)
            .expect("Should restore");
        assert_eq!(restored.player.default_volume, 85);
    }

    #[test]
    fn test_list_backups() {
        let (_temp_dir, manager) = setup_backup_manager();
        let config = Config::default();

        manager.create_backup(&config).expect("Should create");
        std::thread::sleep(std::time::Duration::from_millis(100));
        manager.create_backup(&config).expect("Should create");

        let backups = manager.list_backups().expect("Should list");
        assert_eq!(backups.len(), 2);
    }

    #[test]
    fn test_delete_backup() {
        let (_temp_dir, manager) = setup_backup_manager();
        let config = Config::default();

        let backup_path = manager.create_backup(&config).expect("Should create");
        assert!(backup_path.exists());

        manager.delete_backup(&backup_path).expect("Should delete");
        assert!(!backup_path.exists());
    }

    #[test]
    fn test_delete_all_backups() {
        let (_temp_dir, manager) = setup_backup_manager();
        let config = Config::default();

        manager.create_backup(&config).expect("Should create");
        std::thread::sleep(std::time::Duration::from_millis(100));
        manager.create_backup(&config).expect("Should create");

        let count = manager.delete_all_backups().expect("Should delete all");
        assert_eq!(count, 2);

        let backups = manager.list_backups().expect("Should list");
        assert_eq!(backups.len(), 0);
    }

    #[test]
    fn test_backup_rotation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let backup_dir = temp_dir.path().join("backups");
        let manager = ConfigBackupManager::new(backup_dir).with_max_backups(3);
        let config = Config::default();

        for _ in 0..5 {
            manager.create_backup(&config).expect("Should create");
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let backups = manager.list_backups().expect("Should list");
        assert_eq!(backups.len(), 3);
    }

    #[test]
    fn test_backup_info_formatting() {
        let (_temp_dir, manager) = setup_backup_manager();
        let config = Config::default();

        manager.create_backup(&config).expect("Should create");
        let backups = manager.list_backups().expect("Should list");

        assert!(!backups.is_empty());
        let info = &backups[0];

        let _ = info.created_timestamp();
        let _ = info.size_human();
    }

    #[test]
    fn test_restore_nonexistent_backup() {
        let (_temp_dir, manager) = setup_backup_manager();
        let nonexistent = PathBuf::from("/this/does/not/exist.toml");

        let result = manager.restore_from_backup(&nonexistent);
        assert!(result.is_err());
    }
}
