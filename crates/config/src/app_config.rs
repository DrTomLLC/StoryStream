//! Application-level configuration section

use crate::validation::{ConfigSection, ValidationError, Validator};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Log level for application logging
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

/// Application-level settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AppConfig {
    /// Database file path (relative to config dir if not absolute)
    pub database_path: PathBuf,

    /// Log level for application output
    pub log_level: LogLevel,

    /// Enable debug mode (additional logging and checks)
    pub debug_mode: bool,

    /// Check for updates on startup
    pub check_updates: bool,

    /// Send anonymous usage statistics
    pub telemetry_enabled: bool,

    /// Color scheme preference
    pub color_scheme: ColorScheme,

    /// Maximum number of recent books to track
    pub max_recent_books: usize,

    /// Enable experimental features
    pub experimental_features: bool,
}

/// Color scheme options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    Auto,
    Light,
    Dark,
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorScheme::Auto => write!(f, "auto"),
            ColorScheme::Light => write!(f, "light"),
            ColorScheme::Dark => write!(f, "dark"),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("storystream.db"),
            log_level: LogLevel::Info,
            debug_mode: false,
            check_updates: true,
            telemetry_enabled: false,
            color_scheme: ColorScheme::Auto,
            max_recent_books: 10,
            experimental_features: false,
        }
    }
}

impl ConfigSection for AppConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut results = Vec::new();

        // Validate database path is not empty
        if self.database_path.as_os_str().is_empty() {
            results.push(Err(ValidationError::new(
                "app.database_path",
                "must not be empty",
            )));
        }

        // Validate max_recent_books is reasonable
        results.push(Validator::in_range(
            self.max_recent_books,
            1,
            100,
            "app.max_recent_books",
        ));

        Validator::collect_errors(results)
    }

    fn merge(&mut self, other: Self) {
        self.database_path = other.database_path;
        self.log_level = other.log_level;
        self.debug_mode = other.debug_mode;
        self.check_updates = other.check_updates;
        self.telemetry_enabled = other.telemetry_enabled;
        self.color_scheme = other.color_scheme;
        self.max_recent_books = other.max_recent_books;
        self.experimental_features = other.experimental_features;
    }

    fn section_name(&self) -> &'static str {
        "app"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_valid() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_empty_database_path() {
        let mut config = AppConfig::default();
        config.database_path = PathBuf::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_max_recent_books() {
        let mut config = AppConfig::default();
        config.max_recent_books = 0;
        assert!(config.validate().is_err());

        config.max_recent_books = 150;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_merge() {
        let mut base = AppConfig::default();
        let mut other = AppConfig::default();
        other.log_level = LogLevel::Debug;
        other.debug_mode = true;
        other.max_recent_books = 20;

        base.merge(other);
        assert_eq!(base.log_level, LogLevel::Debug);
        assert!(base.debug_mode);
        assert_eq!(base.max_recent_books, 20);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Error.to_string(), "error");
        assert_eq!(LogLevel::Info.to_string(), "info");
    }

    #[test]
    fn test_color_scheme_display() {
        assert_eq!(ColorScheme::Auto.to_string(), "auto");
        assert_eq!(ColorScheme::Dark.to_string(), "dark");
    }
}