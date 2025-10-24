//! Validation system for configuration values
//!
//! This module provides traits and utilities for validating configuration values.
//! Each config section implements the `ConfigSection` trait which includes validation.

pub use crate::error::ValidationError;
use std::path::Path;

/// Trait for configuration sections that can validate themselves
///
/// Each config section (AppConfig, PlayerConfig, etc.) implements this trait
/// to provide validation logic. This allows the system to be extended with
/// new config sections without modifying existing code.
pub trait ConfigSection: Default {
    /// Validates the configuration section
    ///
    /// Returns a list of validation errors. Empty list means valid.
    fn validate(&self) -> Result<(), Vec<ValidationError>>;

    /// Merges another config section into this one
    ///
    /// Values from `other` take precedence. This is used for override chains.
    fn merge(&mut self, other: Self);

    /// Returns the section name for error reporting
    fn section_name(&self) -> &'static str;
}

/// Common validators for config values
pub struct Validator;

impl Validator {
    /// Validates that a numeric value is within a range
    pub fn in_range<T>(value: T, min: T, max: T, field: &str) -> Result<(), ValidationError>
    where
        T: PartialOrd + std::fmt::Display + Copy,
    {
        if value < min || value > max {
            Err(ValidationError::with_value(
                field,
                format!("must be between {} and {}", min, max),
                value,
            ))
        } else {
            Ok(())
        }
    }

    /// Validates that a path exists
    pub fn path_exists(path: &Path, field: &str) -> Result<(), ValidationError> {
        if !path.exists() {
            Err(ValidationError::with_value(
                field,
                "path does not exist",
                path.display(),
            ))
        } else {
            Ok(())
        }
    }

    /// Validates that a path is a directory
    pub fn is_directory(path: &Path, field: &str) -> Result<(), ValidationError> {
        if !path.is_dir() {
            Err(ValidationError::with_value(
                field,
                "path is not a directory",
                path.display(),
            ))
        } else {
            Ok(())
        }
    }

    /// Validates that a string is not empty
    pub fn not_empty(value: &str, field: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            Err(ValidationError::new(field, "must not be empty"))
        } else {
            Ok(())
        }
    }

    /// Validates that a value is one of the allowed options
    pub fn one_of<T>(value: &T, allowed: &[T], field: &str) -> Result<(), ValidationError>
    where
        T: PartialEq + std::fmt::Display,
    {
        if !allowed.contains(value) {
            let allowed_str = allowed
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Err(ValidationError::with_value(
                field,
                format!("must be one of: {}", allowed_str),
                value,
            ))
        } else {
            Ok(())
        }
    }

    /// Collects multiple validation results into a single result
    pub fn collect_errors(
        results: Vec<Result<(), ValidationError>>,
    ) -> Result<(), Vec<ValidationError>> {
        let errors: Vec<ValidationError> = results.into_iter().filter_map(|r| r.err()).collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_in_range_valid() {
        assert!(Validator::in_range(50, 0, 100, "test").is_ok());
        assert!(Validator::in_range(0, 0, 100, "test").is_ok());
        assert!(Validator::in_range(100, 0, 100, "test").is_ok());
    }

    #[test]
    fn test_in_range_invalid() {
        assert!(Validator::in_range(-1, 0, 100, "test").is_err());
        assert!(Validator::in_range(101, 0, 100, "test").is_err());
    }

    #[test]
    fn test_not_empty_valid() {
        assert!(Validator::not_empty("hello", "test").is_ok());
        assert!(Validator::not_empty("  hello  ", "test").is_ok());
    }

    #[test]
    fn test_not_empty_invalid() {
        assert!(Validator::not_empty("", "test").is_err());
        assert!(Validator::not_empty("   ", "test").is_err());
    }

    #[test]
    fn test_one_of_valid() {
        assert!(Validator::one_of(&"red", &["red", "green", "blue"], "test").is_ok());
    }

    #[test]
    fn test_one_of_invalid() {
        assert!(Validator::one_of(&"yellow", &["red", "green", "blue"], "test").is_err());
    }

    #[test]
    fn test_collect_errors_all_ok() {
        let results = vec![Ok(()), Ok(()), Ok(())];
        assert!(Validator::collect_errors(results).is_ok());
    }

    #[test]
    fn test_collect_errors_some_err() {
        let results = vec![
            Ok(()),
            Err(ValidationError::new("field1", "error1")),
            Ok(()),
            Err(ValidationError::new("field2", "error2")),
        ];
        let result = Validator::collect_errors(results);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 2);
    }

    #[test]
    fn test_path_exists_nonexistent() {
        let path = PathBuf::from("/this/path/definitely/does/not/exist");
        assert!(Validator::path_exists(&path, "test").is_err());
    }
}
