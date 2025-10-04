// FILE: src/speed.rs
// ============================================================================

use crate::{EngineError, EngineResult};

/// Playback speed with validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackSpeed {
    value: f32,
}

impl PlaybackSpeed {
    /// Valid speed range
    pub const MIN: f32 = 0.5;
    pub const MAX: f32 = 3.0;

    /// Creates a new playback speed
    pub fn new(speed: f32) -> EngineResult<Self> {
        if !(Self::MIN..=Self::MAX).contains(&speed) {
            return Err(EngineError::InvalidSpeed(speed));
        }
        Ok(Self { value: speed })
    }

    /// Returns the speed value
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Returns whether pitch correction should be applied
    pub fn needs_pitch_correction(&self) -> bool {
        (self.value - 1.0).abs() > 0.01
    }
}

impl Default for PlaybackSpeed {
    fn default() -> Self {
        Self { value: 1.0 }
    }
}

#[cfg(test)]
mod speed_tests {
    use super::*;

    #[test]
    fn test_valid_speed() {
        assert!(PlaybackSpeed::new(1.0).is_ok());
        assert!(PlaybackSpeed::new(1.5).is_ok());
        assert!(PlaybackSpeed::new(0.5).is_ok());
        assert!(PlaybackSpeed::new(3.0).is_ok());
    }

    #[test]
    fn test_invalid_speed() {
        assert!(PlaybackSpeed::new(0.4).is_err());
        assert!(PlaybackSpeed::new(3.1).is_err());
        assert!(PlaybackSpeed::new(-1.0).is_err());
    }

    #[test]
    fn test_speed_value() {
        let speed = PlaybackSpeed::new(1.5).unwrap();
        assert_eq!(speed.value(), 1.5);
    }

    #[test]
    fn test_pitch_correction() {
        let normal = PlaybackSpeed::default();
        assert!(!normal.needs_pitch_correction());

        let fast = PlaybackSpeed::new(1.5).unwrap();
        assert!(fast.needs_pitch_correction());
    }
}
