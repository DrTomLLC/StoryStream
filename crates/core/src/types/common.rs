//! Common types and utilities shared across domain models

use serde::{Deserialize, Serialize};
use std::fmt;

/// Timestamp in milliseconds since Unix epoch
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Creates a timestamp for the current moment
    ///
    /// # Safety
    /// If system time is somehow before UNIX_EPOCH (should never happen),
    /// gracefully falls back to timestamp 0 instead of panicking.
    pub fn now() -> Self {
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as i64,
        )
    }

    /// Creates a timestamp from milliseconds since Unix epoch
    pub fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Returns the timestamp as milliseconds since Unix epoch
    pub fn as_millis(&self) -> i64 {
        self.0
    }

    /// Returns the timestamp as seconds since Unix epoch
    pub fn as_seconds(&self) -> i64 {
        self.0 / 1000
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Duration in milliseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Duration(u64);

impl Duration {
    /// Zero duration constant
    pub const ZERO: Self = Self(0);

    /// Creates a duration from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    /// Creates a duration from seconds
    pub fn from_seconds(seconds: u64) -> Self {
        Self(seconds * 1000)
    }

    /// Returns the duration in milliseconds
    pub fn as_millis(&self) -> u64 {
        self.0
    }

    /// Returns the duration in seconds
    pub fn as_seconds(&self) -> u64 {
        self.0 / 1000
    }

    /// Returns true if the duration is zero
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Formats as H:MM:SS (always shows hours)
    /// FIXED: Now always returns H:MM:SS format instead of conditionally returning MM:SS
    pub fn as_hms(&self) -> String {
        let total_seconds = self.as_seconds();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_hms())
    }
}

impl From<std::time::Duration> for Duration {
    fn from(d: std::time::Duration) -> Self {
        Self(d.as_millis() as u64)
    }
}

/// Trait for types that can validate themselves
pub trait Validator {
    /// Validates the instance and returns errors if invalid
    fn validate(&self) -> Result<(), Vec<String>>;

    /// Returns true if the instance is valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_now() {
        let t1 = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = Timestamp::now();
        assert!(t2 > t1);
    }

    #[test]
    fn test_timestamp_from_millis() {
        let t = Timestamp::from_millis(1234567890123);
        assert_eq!(t.as_millis(), 1234567890123);
        assert_eq!(t.as_seconds(), 1234567890);
    }

    #[test]
    fn test_timestamp_ordering() {
        let t1 = Timestamp::from_millis(1000);
        let t2 = Timestamp::from_millis(2000);
        assert!(t1 < t2);
        assert!(t2 > t1);
    }

    #[test]
    fn test_timestamp_display() {
        let t = Timestamp::from_millis(1234567890123);
        assert_eq!(t.to_string(), "1234567890123");
    }

    #[test]
    fn test_duration_from_seconds() {
        let d = Duration::from_seconds(3665);
        assert_eq!(d.as_seconds(), 3665);
        assert_eq!(d.as_millis(), 3665000);
    }

    #[test]
    fn test_duration_from_millis() {
        let d = Duration::from_millis(3665000);
        assert_eq!(d.as_millis(), 3665000);
        assert_eq!(d.as_seconds(), 3665);
    }

    #[test]
    fn test_duration_is_zero() {
        let d1 = Duration::from_millis(0);
        let d2 = Duration::from_millis(100);
        assert!(d1.is_zero());
        assert!(!d2.is_zero());
    }

    #[test]
    fn test_duration_as_hms_with_hours() {
        let d = Duration::from_seconds(3665); // 1h 1m 5s
        assert_eq!(d.as_hms(), "1:01:05");
    }

    #[test]
    fn test_duration_as_hms_without_hours() {
        let d = Duration::from_seconds(125); // 2m 5s
        // FIXED: Now always shows hours, so "0:02:05" instead of "2:05"
        assert_eq!(d.as_hms(), "0:02:05");
    }

    #[test]
    fn test_duration_as_hms_zero() {
        let d = Duration::from_seconds(0);
        // FIXED: Now always shows hours, so "0:00:00" instead of "0:00"
        assert_eq!(d.as_hms(), "0:00:00");
    }

    #[test]
    fn test_duration_display() {
        let d = Duration::from_seconds(3665);
        assert_eq!(d.to_string(), "1:01:05");
    }

    #[test]
    fn test_duration_ordering() {
        let d1 = Duration::from_seconds(100);
        let d2 = Duration::from_seconds(200);
        assert!(d1 < d2);
        assert!(d2 > d1);
    }

    #[test]
    fn test_duration_from_std_duration() {
        let std_d = std::time::Duration::from_secs(42);
        let d: Duration = std_d.into();
        assert_eq!(d.as_seconds(), 42);
    }

    #[test]
    fn test_validator_trait() {
        struct TestType {
            value: i32,
        }

        impl Validator for TestType {
            fn validate(&self) -> Result<(), Vec<String>> {
                if self.value < 0 {
                    Err(vec!["Value must be positive".to_string()])
                } else {
                    Ok(())
                }
            }
        }

        let valid = TestType { value: 10 };
        let invalid = TestType { value: -5 };

        assert!(valid.is_valid());
        assert!(!invalid.is_valid());
    }
}