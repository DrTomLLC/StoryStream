// FILE: crates/config/src/migration.rs
//! Configuration migration system
//!
//! This module handles upgrading old config file formats to newer versions.
//! When CONFIG_VERSION is incremented, add a migration function here.

use crate::{Config, ConfigError, ConfigResult, CONFIG_VERSION};

/// Trait for config migrations
pub trait Migration {
    /// Returns the version this migration upgrades TO
    ///
    /// Note: This method is part of the public API and will be used
    /// when the migration system is fully implemented. Currently unused
    /// but kept for API stability.
    #[allow(dead_code)]
    fn target_version(&self) -> u32;

    /// Performs the migration
    fn migrate(&self, value: &mut toml::Value) -> ConfigResult<()>;
}

/// Migrates a config from its current version to the latest version
pub fn migrate_to_latest(config: Config) -> ConfigResult<Config> {
    if config.version == CONFIG_VERSION {
        return Ok(config);
    }

    if config.version > CONFIG_VERSION {
        log::warn!(
            "Config version {} is newer than supported version {}. Attempting to use as-is.",
            config.version,
            CONFIG_VERSION
        );
        return Ok(config);
    }

    log::info!(
        "Migrating config from version {} to {}",
        config.version,
        CONFIG_VERSION
    );

    // Convert to TOML Value for manipulation
    let mut value = toml::to_string(&config)
        .map_err(ConfigError::SerializeError)
        .and_then(|s| {
            toml::from_str(&s).map_err(|e| ConfigError::ValidationError(e.to_string()))
        })?;

    // Apply migrations in sequence
    let mut current_version = config.version;

    while current_version < CONFIG_VERSION {
        let next_version = current_version + 1;

        if let Some(migration) = get_migration(next_version) {
            migration.migrate(&mut value)?;
            log::info!("Applied migration to version {}", next_version);
        } else {
            log::warn!(
                "No migration defined for version {}, skipping",
                next_version
            );
        }

        current_version = next_version;
    }

    // Update version number
    if let Some(table) = value.as_table_mut() {
        table.insert(
            "version".to_string(),
            toml::Value::Integer(CONFIG_VERSION as i64),
        );
    }

    // Convert back to Config
    let migrated: Config =
        toml::from_str(&toml::to_string(&value).map_err(ConfigError::SerializeError)?)
            .map_err(|e| ConfigError::ValidationError(e.to_string()))?;

    Ok(migrated)
}

/// Returns the migration for a specific version, if one exists
fn get_migration(version: u32) -> Option<Box<dyn Migration>> {
    match version {
        // Example migrations for future versions:
        // 2 => Some(Box::new(MigrationV2)),
        // 3 => Some(Box::new(MigrationV3)),
        _ => None,
    }
}

// Example migration (for when version 2 is needed):
// struct MigrationV2;
//
// impl Migration for MigrationV2 {
//     fn target_version(&self) -> u32 {
//         2
//     }
//
//     fn migrate(&self, value: &mut toml::Value) -> ConfigResult<()> {
//         // Example: Rename a field
//         if let Some(table) = value.as_table_mut() {
//             if let Some(player) = table.get_mut("player") {
//                 if let Some(player_table) = player.as_table_mut() {
//                     // Rename old_field to new_field
//                     if let Some(old_value) = player_table.remove("old_field") {
//                         player_table.insert("new_field".to_string(), old_value);
//                     }
//                 }
//             }
//         }
//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_same_version() {
        let config = Config::default();
        let result = migrate_to_latest(config.clone());
        assert!(result.is_ok());
        let migrated = result.expect("Should migrate");
        assert_eq!(migrated.version, config.version);
    }

    #[test]
    fn test_migrate_newer_version() {
        let mut config = Config::default();
        config.version = CONFIG_VERSION + 1;

        let result = migrate_to_latest(config.clone());
        assert!(result.is_ok());
        let migrated = result.expect("Should handle newer version");
        assert_eq!(migrated.version, CONFIG_VERSION + 1);
    }

    #[test]
    fn test_config_to_toml_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).expect("Should serialize");
        let parsed: Config = toml::from_str(&toml_str).expect("Should deserialize");
        assert_eq!(parsed, config);
    }

    #[test]
    fn test_get_migration_returns_none() {
        // Currently no migrations defined
        assert!(get_migration(2).is_none());
        assert!(get_migration(999).is_none());
    }
}
