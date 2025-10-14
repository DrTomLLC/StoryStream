//! Playback speed control

use crate::error::EngineError;

/// Playback speed multiplier
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackSpeed {
    value: f32,
}

impl PlaybackSpeed {
    /// Minimum allowed speed (0.5x)
    pub const MIN: f32 = 0.5;

    /// Maximum allowed speed (3.0x)
    pub const MAX: f32 = 3.0;

    /// Normal speed (1.0x)
    pub const NORMAL: f32 = 1.0;

    /// Creates a new playback speed
    ///
    /// # Errors
    ///
    /// Returns `InvalidSpeed` if speed is outside the range [0.5, 3.0]
    pub fn new(speed: f32) -> Result<Self, EngineError> {
        if !(Self::MIN..=Self::MAX).contains(&speed) {
            return Err(EngineError::InvalidSpeed(speed));
        }
        Ok(Self { value: speed })
    }

    /// Returns the speed value
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Returns true if this is normal speed
    pub fn is_normal(&self) -> bool {
        (self.value - Self::NORMAL).abs() < f32::EPSILON
    }

    /// Returns true if this is slower than normal
    pub fn is_slower(&self) -> bool {
        self.value < Self::NORMAL
    }

    /// Returns true if this is faster than normal
    pub fn is_faster(&self) -> bool {
        self.value > Self::NORMAL
    }
}

impl Default for PlaybackSpeed {
    fn default() -> Self {
        Self { value: Self::NORMAL }
    }
}

impl std::fmt::Display for PlaybackSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}x", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_speeds() {
        assert!(PlaybackSpeed::new(0.5).is_ok());
        assert!(PlaybackSpeed::new(1.0).is_ok());
        assert!(PlaybackSpeed::new(1.5).is_ok());
        assert!(PlaybackSpeed::new(3.0).is_ok());
    }

    #[test]
    fn test_invalid_speeds() {
        assert!(PlaybackSpeed::new(0.4).is_err());
        assert!(PlaybackSpeed::new(3.1).is_err());
        assert!(PlaybackSpeed::new(-1.0).is_err());
    }

    #[test]
    fn test_default_speed() {
        let speed = PlaybackSpeed::default();
        assert_eq!(speed.value(), 1.0);
        assert!(speed.is_normal());
    }

    #[test]
    fn test_speed_comparison() {
        let slow = PlaybackSpeed::new(0.75).unwrap();
        let fast = PlaybackSpeed::new(1.5).unwrap();

        assert!(slow.is_slower());
        assert!(!slow.is_faster());
        assert!(fast.is_faster());
        assert!(!fast.is_slower());
    }

    #[test]
    fn test_display() {
        let speed = PlaybackSpeed::new(1.5).unwrap();
        assert_eq!(format!("{}", speed), "1.50x");
    }
}