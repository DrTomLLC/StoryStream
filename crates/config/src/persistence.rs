//! File system persistence for configuration
//!
//! This module handles reading and writing config files with:
//! - Atomic writes (no partial/corrupted files)
//! - Automatic backups before overwrites
//! - Directory creation
//! - Graceful error handling
//! - NO PANICS - all errors are handled via Result types

use crate::{Config, ConfigError, ConfigResult, CONFIG_VERSION};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Handles configuration file persistence
pub struct ConfigPersistence {
    config_path: PathBuf,
}

impl ConfigPersistence {
    /// Creates a new persistence handler for the given config file path
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Loads configuration from file
    ///
    /// If the file doesn't exist, returns the default config.
    /// If the file is empty or corrupted, returns an error.
    pub fn load(&self) -> ConfigResult<Config> {
        if !self.config_path.exists() {
            log::info!(
                "Config file not found at {}, using defaults",
                self.config_path.display()
            );
            return Ok(Config::default());
        }

        let contents =
            fs::read_to_string(&self.config_path).map_err(|e| ConfigError::ReadError {
                path: self.config_path.clone(),
                source: e,
            })?;

        // CRITICAL: Check for empty or whitespace-only files
        // These are treated as corrupted, not as valid defaults
        if contents.trim().is_empty() {
            return Err(ConfigError::ReadError {
                path: self.config_path.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Config file is empty or contains only whitespace",
                ),
            });
        }

        let mut config: Config =
            toml::from_str(&contents).map_err(|e| ConfigError::ParseError {
                path: self.config_path.clone(),
                source: e,
            })?;

        // Migrate if needed
        if config.version < CONFIG_VERSION {
            log::info!(
                "Config version {} is older than current version {}, migrating...",
                config.version,
                CONFIG_VERSION
            );
            config = crate::migration::migrate_to_latest(config)?;

            // Save migrated config
            log::info!("Saving migrated config");
            self.save(&config)?;
        }

        // Validate the loaded config
        if let Err(errors) = config.validate() {
            let error_msg = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            log::warn!("Config validation warnings: {}", error_msg);
            // Don't fail on validation errors, just warn
            // This allows users to fix invalid configs without losing data
        }

        Ok(config)
    }

    /// Saves configuration to file atomically
    ///
    /// This uses a temporary file and atomic rename to ensure the config
    /// file is never left in a corrupted state.
    pub fn save(&self, config: &Config) -> ConfigResult<()> {
        // Validate before saving
        if let Err(errors) = config.validate() {
            let error_msg = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(ConfigError::ValidationError(error_msg));
        }

        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            self.ensure_directory_exists(parent)?;
        }

        // Backup existing config if it exists
        if self.config_path.exists() {
            self.backup_config()?;
        }

        // Serialize config to TOML
        let toml_string = toml::to_string_pretty(config).map_err(ConfigError::SerializeError)?;

        // Write to temporary file first
        let temp_file = self.create_temp_file()?;
        self.write_atomic(temp_file, &toml_string)?;

        log::info!("Config saved to {}", self.config_path.display());
        Ok(())
    }

    /// Ensures a directory exists, creating it if necessary
    fn ensure_directory_exists(&self, path: &Path) -> ConfigResult<()> {
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| ConfigError::DirectoryCreationError {
                path: path.to_path_buf(),
                source: e,
            })?;
            log::info!("Created config directory: {}", path.display());
        }
        Ok(())
    }

    /// Creates a backup of the current config file
    fn backup_config(&self) -> ConfigResult<()> {
        let backup_path = self.config_path.with_extension("toml.backup");
        fs::copy(&self.config_path, &backup_path)
            .map_err(|e| ConfigError::BackupError { source: e })?;
        log::debug!("Backed up config to {}", backup_path.display());
        Ok(())
    }

    /// Creates a temporary file in the same directory as the config file
    fn create_temp_file(&self) -> ConfigResult<NamedTempFile> {
        let dir = self
            .config_path
            .parent()
            .ok_or_else(|| ConfigError::PathResolutionError {
                reason: "Config path has no parent directory".to_string(),
            })?;

        NamedTempFile::new_in(dir).map_err(ConfigError::IoError)
    }

    /// Writes content to a temporary file and atomically renames it
    fn write_atomic(&self, mut temp_file: NamedTempFile, content: &str) -> ConfigResult<()> {
        // Write content to temp file
        temp_file
            .write_all(content.as_bytes())
            .map_err(ConfigError::IoError)?;

        // Flush to ensure all data is written
        temp_file.flush().map_err(ConfigError::IoError)?;

        // Atomically rename temp file to target path
        temp_file
            .persist(&self.config_path)
            .map_err(|e| ConfigError::WriteError {
                path: self.config_path.clone(),
                source: e.error,
            })?;

        Ok(())
    }

    /// Generates a default config file with comments
    pub fn generate_default_with_comments(&self) -> ConfigResult<()> {
        let default_config = Config::default();

        // For now, just save the default config
        // In the future, we could add comments using toml_edit crate
        self.save(&default_config)?;

        log::info!("Generated default config at {}", self.config_path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_dir() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        (temp_dir, config_path)
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let (_temp_dir, config_path) = setup_test_dir();
        let persistence = ConfigPersistence::new(config_path.clone());

        let config = persistence.load().expect("Should load default config");
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let (_temp_dir, config_path) = setup_test_dir();
        let persistence = ConfigPersistence::new(config_path.clone());

        let mut config = Config::default();
        config.player.default_volume = 85;

        persistence.save(&config).expect("Should save config");
        let loaded = persistence.load().expect("Should load config");

        assert_eq!(loaded.player.default_volume, 85);
    }

    #[test]
    fn test_save_creates_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("subdir").join("config.toml");
        let persistence = ConfigPersistence::new(config_path.clone());

        let config = Config::default();
        persistence
            .save(&config)
            .expect("Should create directory and save");

        assert!(config_path.exists());
    }

    #[test]
    fn test_backup_created_on_overwrite() {
        let (_temp_dir, config_path) = setup_test_dir();
        let persistence = ConfigPersistence::new(config_path.clone());

        // Save initial config
        let config = Config::default();
        persistence.save(&config).expect("Should save config");

        // Save again to trigger backup
        persistence.save(&config).expect("Should save config again");

        let backup_path = config_path.with_extension("toml.backup");
        assert!(backup_path.exists());
    }

    #[test]
    fn test_invalid_config_returns_error() {
        let (_temp_dir, config_path) = setup_test_dir();

        // Write invalid TOML
        fs::write(&config_path, "this is not valid TOML {{{").expect("Should write file");

        let persistence = ConfigPersistence::new(config_path);
        let result = persistence.load();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::ParseError { .. }
        ));
    }

    #[test]
    fn test_validate_before_save() {
        let (_temp_dir, config_path) = setup_test_dir();
        let persistence = ConfigPersistence::new(config_path);

        let mut config = Config::default();
        config.player.default_volume = 150; // Invalid

        let result = persistence.save(&config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::ValidationError(_)
        ));
    }

    #[test]
    fn test_generate_default_with_comments() {
        let (_temp_dir, config_path) = setup_test_dir();
        let persistence = ConfigPersistence::new(config_path.clone());

        persistence
            .generate_default_with_comments()
            .expect("Should generate default config");

        assert!(config_path.exists());

        let loaded = persistence.load().expect("Should load generated config");
        assert_eq!(loaded, Config::default());
    }
}
