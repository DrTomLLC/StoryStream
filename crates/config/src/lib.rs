//! StoryStream Configuration System
//!
//! This module provides a bulletproof, extensible configuration system.
//! New features can add config sections by implementing the `ConfigSection` trait.
//!
//! # Architecture
//!
//! - **Trait-based**: Each feature defines its config as a type implementing `ConfigSection`
//! - **Graceful degradation**: Invalid configs fall back to defaults with warnings
//! - **Atomic writes**: Config files are never left in a corrupted state
//! - **Zero panics**: All errors are handled via Result types
//!
//! # Example
//!
//! ```rust
//! use storystream_config::{Config, ConfigManager};
//!
//! // Load config (creates default if missing)
//! let manager = ConfigManager::new().expect("Failed to initialize config");
//! let config = manager.load().unwrap_or_else(|e| {
//!     eprintln!("Config error: {}, using defaults", e);
//!     Config::default()
//! });
//!
//! // Use config
//! println!("Volume: {}", config.player.default_volume);
//! ```

mod error;
mod manager;
mod migration;
mod persistence;
mod validation;

// Optional features
pub mod backup;
pub mod schema;
pub mod watcher;

// Config sections
pub mod app_config;
mod library_config;
mod player_config;

pub use error::{ConfigError, ConfigResult, ValidationError}; // Add ValidationError here
pub use manager::ConfigManager;
pub use validation::{ConfigSection, Validator}; // Remove ValidationError from here

// Re-export config sections
pub use app_config::AppConfig;
pub use library_config::LibraryConfig;
pub use player_config::PlayerConfig;

use serde::{Deserialize, Serialize};

/// Current config file format version for migrations
pub const CONFIG_VERSION: u32 = 1;

/// Root configuration structure
///
/// This contains all config sections. New sections can be added here,
/// and they will automatically be included in load/save operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// Config file format version
    pub version: u32,

    /// Application-level settings
    pub app: AppConfig,

    /// Player preferences
    pub player: PlayerConfig,

    /// Library and import settings
    pub library: LibraryConfig,
}

impl Config {
    /// Creates a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates the entire configuration
    ///
    /// Returns all validation errors found across all sections.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Err(mut e) = self.app.validate() {
            errors.append(&mut e);
        }

        if let Err(mut e) = self.player.validate() {
            errors.append(&mut e);
        }

        if let Err(mut e) = self.library.validate() {
            errors.append(&mut e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Merges this config with another, preferring values from `other`
    ///
    /// This is used for override chains: defaults < file < env vars < CLI args
    pub fn merge(&mut self, other: Config) {
        self.app.merge(other.app);
        self.player.merge(other.player);
        self.library.merge(other.library);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            app: AppConfig::default(),
            player: PlayerConfig::default(),
            library: LibraryConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_version_is_set() {
        let config = Config::default();
        assert_eq!(config.version, CONFIG_VERSION);
    }

    #[test]
    fn test_config_merge() {
        let mut base = Config::default();
        let mut override_config = Config::default();
        override_config.player.default_volume = 75;

        base.merge(override_config);
        assert_eq!(base.player.default_volume, 75);
    }
}
