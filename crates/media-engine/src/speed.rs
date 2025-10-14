//! Playback speed control with validation

use crate::{EngineError, Result};
use std::fmt;

/// Playback speed multiplier (0.5x - 3.0x)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Speed(f32);

impl Speed {
    pub const MIN: f32 = 0.5;
    pub const MAX: f32 = 3.0;
    pub const NORMAL: f32 = 1.0;

    pub fn new(speed: f32) -> Result<Self> {
        if speed < Self::MIN || speed > Self::MAX {
            return Err(EngineError::InvalidSpeed(speed));
        }
        if !speed.is_finite() {
            return Err(EngineError::InvalidSpeed(speed));
        }
        Ok(Speed(speed))
    }

    pub fn get(&self) -> f32 {
        self.0
    }

    pub fn is_normal(&self) -> bool {
        (self.0 - Self::NORMAL).abs() < f32::EPSILON
    }

    pub fn is_slower(&self) -> bool {
        self.0 < Self::NORMAL
    }

    pub fn is_faster(&self) -> bool {
        self.0 > Self::NORMAL
    }
}

impl Default for Speed {
    fn default() -> Self {
        Speed(Self::NORMAL)
    }
}

impl fmt::Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}x", self.0)
    }
}

impl TryFrom<f32> for Speed {
    type Error = EngineError;

    fn try_from(value: f32) -> Result<Self> {
        Speed::new(value)
    }
}

impl From<Speed> for f32 {
    fn from(speed: Speed) -> Self {
        speed.0
    }
}

pub struct SpeedProcessor {
    sample_rate: u32,
    channels: u16,
    speed: f32,
}

impl SpeedProcessor {
    pub fn new(sample_rate: u32, channels: u16, speed: f32) -> Result<Self> {
        if !(0.5..=3.0).contains(&speed) || !speed.is_finite() {
            return Err(EngineError::InvalidSpeed(speed));
        }
        Ok(Self { sample_rate, channels, speed })
    }

    pub fn process(&mut self, input: &[f32]) -> Result<Vec<f32>> {
        Ok(input.to_vec())
    }

    pub fn set_speed(&mut self, speed: f32) -> Result<()> {
        if !(0.5..=3.0).contains(&speed) || !speed.is_finite() {
            return Err(EngineError::InvalidSpeed(speed));
        }
        self.speed = speed;
        Ok(())
    }

    pub fn current_speed(&self) -> f32 { self.speed }
    pub fn target_speed(&self) -> f32 { self.speed }
    pub fn set_pitch_correction(&mut self, _enabled: bool) {}
    pub fn flush(&mut self) -> Result<Vec<f32>> { Ok(Vec::new()) }
    pub fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_speeds() {
        assert!(Speed::new(0.5).is_ok());
        assert!(Speed::new(1.0).is_ok());
        assert!(Speed::new(1.5).is_ok());
        assert!(Speed::new(2.0).is_ok());
        assert!(Speed::new(3.0).is_ok());
    }

    #[test]
    fn test_invalid_speeds() {
        assert!(Speed::new(0.4).is_err());
        assert!(Speed::new(3.1).is_err());
        assert!(Speed::new(0.0).is_err());
        assert!(Speed::new(-1.0).is_err());
        assert!(Speed::new(f32::NAN).is_err());
        assert!(Speed::new(f32::INFINITY).is_err());
    }

    #[test]
    fn test_default_speed() {
        let speed = Speed::default();
        assert_eq!(speed.get(), 1.0);
        assert!(speed.is_normal());
    }

    #[test]
    fn test_speed_comparison() {
        let slow = Speed::new(0.75).unwrap();
        let normal = Speed::new(1.0).unwrap();
        let fast = Speed::new(1.5).unwrap();

        assert!(slow.is_slower());
        assert!(!slow.is_faster());

        assert!(normal.is_normal());
        assert!(!normal.is_slower());
        assert!(!normal.is_faster());

        assert!(fast.is_faster());
        assert!(!fast.is_slower());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Speed::new(0.5).unwrap()), "0.50x");
        assert_eq!(format!("{}", Speed::new(1.0).unwrap()), "1.00x");
        assert_eq!(format!("{}", Speed::new(2.5).unwrap()), "2.50x");
    }
}