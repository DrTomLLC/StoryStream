//! Configuration manager - main API for config operations

use crate::persistence::ConfigPersistence;
use crate::{Config, ConfigError, ConfigResult};
use directories::ProjectDirs;
use std::path::PathBuf;

/// Main configuration manager
///
/// This is the primary interface for loading, saving, and managing configuration.
/// It handles file paths, defaults, and validation.
pub struct ConfigManager {
    persistence: ConfigPersistence,
    config_dir: PathBuf,
}

impl ConfigManager {
    /// Creates a new config manager using the default config directory
    ///
    /// The default directory follows XDG base directory specification:
    /// - Linux: `~/.config/storystream/`
    /// - macOS: `~/Library/Application Support/storystream/`
    /// - Windows: `%APPDATA%\storystream\`
    pub fn new() -> ConfigResult<Self> {
        let config_dir = Self::default_config_dir()?;
        Self::with_directory(config_dir)
    }

    /// Creates a config manager with a custom config directory
    pub fn with_directory(config_dir: PathBuf) -> ConfigResult<Self> {
        let config_path = config_dir.join("config.toml");
        let persistence = ConfigPersistence::new(config_path);

        Ok(Self {
            persistence,
            config_dir,
        })
    }

    /// Returns the default config directory based on the platform
    fn default_config_dir() -> ConfigResult<PathBuf> {
        ProjectDirs::from("", "", "storystream")
            .map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
            .ok_or_else(|| ConfigError::PathResolutionError {
                reason: "Could not determine user config directory".to_string(),
            })
    }

    /// Returns the config directory path
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Returns the full config file path
    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    /// Loads the configuration from file
    ///
    /// If the file doesn't exist, returns default configuration.
    /// If the file is corrupted, returns an error.
    pub fn load(&self) -> ConfigResult<Config> {
        self.persistence.load()
    }

    /// Loads the configuration, falling back to defaults on any error
    ///
    /// This is a convenience method that never returns an error.
    /// Errors are logged but the function always returns a valid config.
    pub fn load_or_default(&self) -> Config {
        match self.load() {
            Ok(config) => config,
            Err(e) => {
                log::warn!("Failed to load config: {}, using defaults", e);
                Config::default()
            }
        }
    }

    /// Saves the configuration to file
    ///
    /// This performs validation before saving and uses atomic writes
    /// to prevent corruption.
    pub fn save(&self, config: &Config) -> ConfigResult<()> {
        self.persistence.save(config)
    }

    /// Updates the configuration using a closure
    ///
    /// This loads the current config, applies the update function,
    /// and saves the result atomically.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use storystream_config::ConfigManager;
    /// # let manager = ConfigManager::new().unwrap();
    /// manager.update(|config| {
    ///     config.player.default_volume = 80;
    /// }).expect("Failed to update config");
    /// ```
    pub fn update<F>(&self, update_fn: F) -> ConfigResult<()>
    where
        F: FnOnce(&mut Config),
    {
        let mut config = self.load()?;
        update_fn(&mut config);
        self.save(&config)
    }

    /// Generates a default config file if one doesn't exist
    ///
    /// Returns Ok(true) if a new file was created, Ok(false) if one already exists.
    pub fn initialize(&self) -> ConfigResult<bool> {
        if self.config_path().exists() {
            log::info!(
                "Config file already exists at {}",
                self.config_path().display()
            );
            return Ok(false);
        }

        self.persistence.generate_default_with_comments()?;
        Ok(true)
    }

    /// Resets the configuration to defaults
    ///
    /// This overwrites the existing config file with default values.
    pub fn reset(&self) -> ConfigResult<()> {
        let default_config = Config::default();
        self.save(&default_config)
    }

    /// Validates the current configuration file
    ///
    /// Returns all validation errors found, or Ok if valid.
    pub fn validate(&self) -> ConfigResult<Vec<String>> {
        let config = self.load()?;

        match config.validate() {
            Ok(()) => Ok(Vec::new()),
            Err(errors) => Ok(errors.iter().map(|e| e.to_string()).collect()),
        }
    }

    /// Merges environment variable overrides into the config
    ///
    /// Environment variables follow the pattern: STORYSTREAM_SECTION_FIELD
    /// Example: STORYSTREAM_PLAYER_DEFAULT_VOLUME=80
    ///
    /// This is useful for containerized deployments or CI/CD environments.
    pub fn load_with_env_overrides(&self) -> ConfigResult<Config> {
        let mut config = self.load()?;

        // Check for environment variable overrides
        // Format: STORYSTREAM_SECTION_FIELD=value

        // Player overrides
        if let Ok(volume) = std::env::var("STORYSTREAM_PLAYER_DEFAULT_VOLUME") {
            if let Ok(v) = volume.parse::<u8>() {
                config.player.default_volume = v;
            }
        }

        if let Ok(speed) = std::env::var("STORYSTREAM_PLAYER_DEFAULT_SPEED") {
            if let Ok(s) = speed.parse::<f32>() {
                config.player.default_speed = s;
            }
        }

        // App overrides
        if let Ok(log_level) = std::env::var("STORYSTREAM_APP_LOG_LEVEL") {
            // This is simplified - in production you'd parse the string to LogLevel enum
            log::info!("Log level override: {}", log_level);
        }

        if let Ok(db_path) = std::env::var("STORYSTREAM_APP_DATABASE_PATH") {
            config.app.database_path = PathBuf::from(db_path);
        }

        // Validate after applying overrides
        if let Err(errors) = config.validate() {
            log::warn!(
                "Config validation warnings after env overrides: {:?}",
                errors
            );
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_manager() -> (TempDir, ConfigManager) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = ConfigManager::with_directory(temp_dir.path().to_path_buf())
            .expect("Failed to create manager");
        (temp_dir, manager)
    }

    #[test]
    fn test_new_manager() {
        let result = ConfigManager::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_or_default_with_missing_file() {
        let (_temp_dir, manager) = setup_test_manager();
        let config = manager.load_or_default();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_save_and_load() {
        let (_temp_dir, manager) = setup_test_manager();

        let mut config = Config::default();
        config.player.default_volume = 75;

        manager.save(&config).expect("Should save config");
        let loaded = manager.load().expect("Should load config");

        assert_eq!(loaded.player.default_volume, 75);
    }

    #[test]
    fn test_update() {
        let (_temp_dir, manager) = setup_test_manager();

        // Save initial config
        manager.save(&Config::default()).expect("Should save");

        // Update using closure
        manager
            .update(|config| {
                config.player.default_volume = 90;
            })
            .expect("Should update");

        let loaded = manager.load().expect("Should load");
        assert_eq!(loaded.player.default_volume, 90);
    }

    #[test]
    fn test_initialize_creates_file() {
        let (_temp_dir, manager) = setup_test_manager();

        let created = manager.initialize().expect("Should initialize");
        assert!(created);
        assert!(manager.config_path().exists());
    }

    #[test]
    fn test_initialize_with_existing_file() {
        let (_temp_dir, manager) = setup_test_manager();

        manager.save(&Config::default()).expect("Should save");

        let created = manager.initialize().expect("Should initialize");
        assert!(!created);
    }

    #[test]
    fn test_reset() {
        let (_temp_dir, manager) = setup_test_manager();

        // Save modified config
        let mut config = Config::default();
        config.player.default_volume = 99;
        manager.save(&config).expect("Should save");

        // Reset to defaults
        manager.reset().expect("Should reset");

        let loaded = manager.load().expect("Should load");
        assert_eq!(loaded, Config::default());
    }

    #[test]
    fn test_validate_valid_config() {
        let (_temp_dir, manager) = setup_test_manager();
        manager.save(&Config::default()).expect("Should save");

        let errors = manager.validate().expect("Should validate");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_config() {
        let (_temp_dir, manager) = setup_test_manager();

        let mut config = Config::default();
        config.player.default_volume = 150; // Invalid
        manager
            .save(&config)
            .expect_err("Should not save invalid config");
    }

    #[test]
    fn test_env_override_volume() {
        let (_temp_dir, manager) = setup_test_manager();
        manager.save(&Config::default()).expect("Should save");

        std::env::set_var("STORYSTREAM_PLAYER_DEFAULT_VOLUME", "85");

        let config = manager
            .load_with_env_overrides()
            .expect("Should load with overrides");
        assert_eq!(config.player.default_volume, 85);

        std::env::remove_var("STORYSTREAM_PLAYER_DEFAULT_VOLUME");
    }

    #[test]
    fn test_config_dir_path() {
        let (_temp_dir, manager) = setup_test_manager();
        assert!(manager.config_dir().is_dir() || !manager.config_dir().exists());
    }

    #[test]
    fn test_config_file_path() {
        let (_temp_dir, manager) = setup_test_manager();
        assert!(manager.config_path().ends_with("config.toml"));
    }
}
