//! Configuration file watcher for hot-reloading
//!
//! This module provides optional hot-reload functionality for long-running processes.
//! When enabled, config changes are automatically detected and reloaded.

use crate::{Config, ConfigError, ConfigResult};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

/// Configuration watcher that detects file changes
///
/// This is useful for long-running processes like servers or daemons
/// that need to reload configuration without restarting.
pub struct ConfigWatcher {
    config_path: PathBuf,
    current_config: Arc<RwLock<Config>>,
    last_modified: SystemTime,
    check_interval: Duration,
}

impl ConfigWatcher {
    /// Creates a new config watcher
    pub fn new(config_path: PathBuf, initial_config: Config) -> ConfigResult<Self> {
        let last_modified = std::fs::metadata(&config_path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        Ok(Self {
            config_path,
            current_config: Arc::new(RwLock::new(initial_config)),
            last_modified,
            check_interval: Duration::from_secs(2),
        })
    }

    /// Sets the check interval for file modifications
    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Returns a handle to the current configuration
    ///
    /// This can be cloned and shared across threads.
    pub fn config_handle(&self) -> Arc<RwLock<Config>> {
        Arc::clone(&self.current_config)
    }

    /// Checks if the config file has been modified and reloads if needed
    ///
    /// Returns true if config was reloaded, false if no changes detected.
    pub fn check_and_reload(&mut self) -> ConfigResult<bool> {
        let metadata = std::fs::metadata(&self.config_path).map_err(|e| ConfigError::IoError(e))?;

        let modified = metadata.modified().map_err(|e| ConfigError::IoError(e))?;

        if modified > self.last_modified {
            log::info!("Config file modified, reloading...");

            match self.reload_config() {
                Ok(()) => {
                    self.last_modified = modified;
                    log::info!("Config reloaded successfully");
                    Ok(true)
                }
                Err(e) => {
                    log::error!("Failed to reload config: {}", e);
                    // Don't update last_modified so we can retry
                    Err(e)
                }
            }
        } else {
            Ok(false)
        }
    }

    /// Reloads the configuration from disk
    fn reload_config(&self) -> ConfigResult<()> {
        let contents = std::fs::read_to_string(&self.config_path).map_err(|e| {
            ConfigError::ReadError {
                path: self.config_path.clone(),
                source: e,
            }
        })?;

        let new_config: Config =
            toml::from_str(&contents).map_err(|e| ConfigError::ParseError {
                path: self.config_path.clone(),
                source: e,
            })?;

        // Validate before applying
        if let Err(errors) = new_config.validate() {
            let error_msg = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            log::warn!("Config validation warnings after reload: {}", error_msg);
            // Continue anyway - validation errors are warnings
        }

        // Update the shared config
        let mut config = self.current_config.write().map_err(|_| {
            ConfigError::ValidationError("Failed to acquire config write lock".to_string())
        })?;

        *config = new_config;

        Ok(())
    }

    /// Starts watching for config changes in a background thread
    ///
    /// Returns a handle that can be used to stop watching.
    pub fn start_watching(mut self) -> WatchHandle {
        let (tx, rx) = std::sync::mpsc::channel();

        let handle = thread::spawn(move || {
            log::info!(
                "Config watcher started for {}",
                self.config_path.display()
            );

            loop {
                // Check for stop signal
                if rx.try_recv().is_ok() {
                    log::info!("Config watcher stopped");
                    break;
                }

                // Check for config changes
                if let Err(e) = self.check_and_reload() {
                    log::error!("Error checking config: {}", e);
                }

                thread::sleep(self.check_interval);
            }
        });

        WatchHandle {
            stop_tx: tx,
            thread_handle: Some(handle),
        }
    }
}

/// Handle for a running config watcher
///
/// Dropping this handle will stop the watcher thread.
pub struct WatchHandle {
    stop_tx: std::sync::mpsc::Sender<()>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl WatchHandle {
    /// Stops the watcher and waits for the thread to finish
    pub fn stop(mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for WatchHandle {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_config() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");

        let config = Config::default();
        let toml_string = toml::to_string(&config).expect("Failed to serialize");
        fs::write(&config_path, toml_string).expect("Failed to write config");

        (temp_dir, config_path)
    }

    #[test]
    fn test_watcher_creation() {
        let (_temp_dir, config_path) = setup_test_config();
        let config = Config::default();

        let watcher = ConfigWatcher::new(config_path, config);
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_check_no_changes() {
        let (_temp_dir, config_path) = setup_test_config();
        let config = Config::default();

        let mut watcher = ConfigWatcher::new(config_path, config).expect("Failed to create watcher");

        let reloaded = watcher.check_and_reload().expect("Check failed");
        assert!(!reloaded);
    }

    #[test]
    fn test_check_with_changes() {
        let (_temp_dir, config_path) = setup_test_config();
        let config = Config::default();

        let mut watcher = ConfigWatcher::new(config_path.clone(), config)
            .expect("Failed to create watcher");

        // Wait a bit to ensure timestamp difference
        thread::sleep(Duration::from_millis(10));

        // Modify config file
        let mut new_config = Config::default();
        new_config.player.default_volume = 85;
        let toml_string = toml::to_string(&new_config).expect("Failed to serialize");
        fs::write(&config_path, toml_string).expect("Failed to write config");

        // Check should detect change
        let reloaded = watcher.check_and_reload().expect("Check failed");
        assert!(reloaded);

        // Verify new value
        let handle = watcher.config_handle();
        let config = handle.read().expect("Failed to read config");
        assert_eq!(config.player.default_volume, 85);
    }

    #[test]
    fn test_config_handle_shared() {
        let (_temp_dir, config_path) = setup_test_config();
        let config = Config::default();

        let watcher = ConfigWatcher::new(config_path, config).expect("Failed to create watcher");

        let handle1 = watcher.config_handle();
        let handle2 = watcher.config_handle();

        // Both handles point to same config
        let config1 = handle1.read().expect("Failed to read");
        let config2 = handle2.read().expect("Failed to read");

        assert_eq!(*config1, *config2);
    }

    #[test]
    fn test_custom_check_interval() {
        let (_temp_dir, config_path) = setup_test_config();
        let config = Config::default();

        let watcher = ConfigWatcher::new(config_path, config)
            .expect("Failed to create watcher")
            .with_check_interval(Duration::from_millis(100));

        assert_eq!(watcher.check_interval, Duration::from_millis(100));
    }

    #[test]
    fn test_watcher_thread_start_stop() {
        let (_temp_dir, config_path) = setup_test_config();
        let config = Config::default();

        let watcher = ConfigWatcher::new(config_path, config)
            .expect("Failed to create watcher")
            .with_check_interval(Duration::from_millis(50));

        let handle = watcher.start_watching();

        // Let it run briefly
        thread::sleep(Duration::from_millis(200));

        // Stop should work
        handle.stop();
    }
}