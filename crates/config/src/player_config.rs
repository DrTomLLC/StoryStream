//! Player configuration section

use crate::validation::{ConfigSection, ValidationError, Validator};
use serde::{Deserialize, Serialize};

/// Player preferences and behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PlayerConfig {
    /// Default volume level (0-100)
    pub default_volume: u8,

    /// Default playback speed (0.5 - 2.0)
    pub default_speed: f32,

    /// Auto-save playback position interval in seconds
    pub autosave_interval_secs: u64,

    /// Resume playback from last position on start
    pub auto_resume: bool,

    /// Skip silence automatically
    pub skip_silence: bool,

    /// Rewind seconds when resuming playback
    pub resume_rewind_secs: u64,

    /// UI refresh rate in milliseconds
    pub ui_refresh_ms: u64,

    /// Volume change step for increment/decrement (0-100)
    pub volume_step: u8,

    /// Playback speed change step
    pub speed_step: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            default_volume: 70,
            default_speed: 1.0,
            autosave_interval_secs: 5,
            auto_resume: true,
            skip_silence: false,
            resume_rewind_secs: 3,
            ui_refresh_ms: 100,
            volume_step: 5,
            speed_step: 0.1,
        }
    }
}

impl ConfigSection for PlayerConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        Validator::collect_errors(vec![
            Validator::in_range(self.default_volume, 0, 100, "player.default_volume"),
            Validator::in_range(self.default_speed, 0.5, 2.0, "player.default_speed"),
            Validator::in_range(
                self.autosave_interval_secs,
                1,
                300,
                "player.autosave_interval_secs",
            ),
            Validator::in_range(self.resume_rewind_secs, 0, 60, "player.resume_rewind_secs"),
            Validator::in_range(self.ui_refresh_ms, 16, 1000, "player.ui_refresh_ms"),
            Validator::in_range(self.volume_step, 1, 50, "player.volume_step"),
            Validator::in_range(self.speed_step, 0.05, 0.5, "player.speed_step"),
        ])
    }

    fn merge(&mut self, other: Self) {
        self.default_volume = other.default_volume;
        self.default_speed = other.default_speed;
        self.autosave_interval_secs = other.autosave_interval_secs;
        self.auto_resume = other.auto_resume;
        self.skip_silence = other.skip_silence;
        self.resume_rewind_secs = other.resume_rewind_secs;
        self.ui_refresh_ms = other.ui_refresh_ms;
        self.volume_step = other.volume_step;
        self.speed_step = other.speed_step;
    }

    fn section_name(&self) -> &'static str {
        "player"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_valid() {
        let config = PlayerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_volume() {
        let mut config = PlayerConfig::default();
        config.default_volume = 101;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_speed() {
        let mut config = PlayerConfig::default();
        config.default_speed = 3.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_autosave_interval() {
        let mut config = PlayerConfig::default();
        config.autosave_interval_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_merge() {
        let mut base = PlayerConfig::default();
        let mut other = PlayerConfig::default();
        other.default_volume = 80;
        other.auto_resume = false;

        base.merge(other);
        assert_eq!(base.default_volume, 80);
        assert!(!base.auto_resume);
    }

    #[test]
    fn test_multiple_validation_errors() {
        let config = PlayerConfig {
            default_volume: 101,
            default_speed: 3.0,
            autosave_interval_secs: 0,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 3);
    }
}
