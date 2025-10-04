//! Playback-related domain models

use crate::types::{BookId, Duration, Timestamp, Validator};
use serde::{Deserialize, Serialize};

/// Represents the current playback state for a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub book_id: BookId,
    pub position: Duration,
    pub speed: PlaybackSpeed,
    pub volume: u8, // 0-100
    pub is_playing: bool,
    pub equalizer: Option<EqualizerPreset>,
    pub sleep_timer: Option<SleepTimer>,
    pub skip_silence: bool,
    pub volume_boost: u8, // 0-100, additional boost for quiet recordings
    pub last_updated: Timestamp,
}

impl PlaybackState {
    /// Creates a new playback state for a book
    pub fn new(book_id: BookId) -> Self {
        Self {
            book_id,
            position: Duration::from_millis(0),
            speed: PlaybackSpeed::default(),
            volume: 100,
            is_playing: false,
            equalizer: None,
            sleep_timer: None,
            skip_silence: false,
            volume_boost: 0,
            last_updated: Timestamp::now(),
        }
    }

    /// Updates the playback position
    pub fn set_position(&mut self, position: Duration) {
        self.position = position;
        self.last_updated = Timestamp::now();
    }

    /// Starts playback
    pub fn play(&mut self) {
        self.is_playing = true;
        self.last_updated = Timestamp::now();
    }

    /// Pauses playback
    pub fn pause(&mut self) {
        self.is_playing = false;
        self.last_updated = Timestamp::now();
    }

    /// Sets the playback speed
    pub fn set_speed(&mut self, speed: PlaybackSpeed) {
        self.speed = speed;
        self.last_updated = Timestamp::now();
    }
}

impl Validator for PlaybackState {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.volume > 100 {
            errors.push("Volume must be between 0 and 100".to_string());
        }

        if self.volume_boost > 100 {
            errors.push("Volume boost must be between 0 and 100".to_string());
        }

        if let Err(speed_errors) = self.speed.validate() {
            errors.extend(speed_errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Playback speed with pitch correction
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlaybackSpeed {
    speed: f32,
    pitch_correction: bool,
}

impl PlaybackSpeed {
    /// Creates a new playback speed (0.5x - 3.0x)
    pub fn new(speed: f32) -> Result<Self, String> {
        if !(0.5..=3.0).contains(&speed) {
            Err("Speed must be between 0.5 and 3.0".to_string())
        } else {
            Ok(Self {
                speed,
                pitch_correction: true,
            })
        }
    }

    /// Creates a playback speed without validation (for deserialization)
    pub fn new_unchecked(speed: f32) -> Self {
        Self {
            speed,
            pitch_correction: true,
        }
    }

    /// Returns the speed value
    pub fn value(&self) -> f32 {
        self.speed
    }

    /// Returns true if pitch correction is enabled
    pub fn has_pitch_correction(&self) -> bool {
        self.pitch_correction
    }

    /// Sets pitch correction
    pub fn with_pitch_correction(mut self, enabled: bool) -> Self {
        self.pitch_correction = enabled;
        self
    }
}

impl Default for PlaybackSpeed {
    fn default() -> Self {
        Self {
            speed: 1.0,
            pitch_correction: true,
        }
    }
}

impl Validator for PlaybackSpeed {
    fn validate(&self) -> Result<(), Vec<String>> {
        if !(0.5..=3.0).contains(&self.speed) {
            Err(vec!["Speed must be between 0.5 and 3.0".to_string()])
        } else {
            Ok(())
        }
    }
}

/// 10-band equalizer preset
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqualizerPreset {
    pub name: String,
    pub bands: [EqualizerBand; 10],
}

impl EqualizerPreset {
    /// Creates a flat equalizer (all bands at 0dB)
    pub fn flat() -> Self {
        Self {
            name: "Flat".to_string(),
            bands: [EqualizerBand::default(); 10],
        }
    }

    /// Creates a bass boost preset
    pub fn bass_boost() -> Self {
        Self {
            name: "Bass Boost".to_string(),
            bands: [
                EqualizerBand::new(32, 6.0),
                EqualizerBand::new(64, 4.0),
                EqualizerBand::new(125, 2.0),
                EqualizerBand::new(250, 0.0),
                EqualizerBand::new(500, 0.0),
                EqualizerBand::new(1000, 0.0),
                EqualizerBand::new(2000, 0.0),
                EqualizerBand::new(4000, 0.0),
                EqualizerBand::new(8000, 0.0),
                EqualizerBand::new(16000, 0.0),
            ],
        }
    }

    /// Creates a voice boost preset
    pub fn voice_boost() -> Self {
        Self {
            name: "Voice Boost".to_string(),
            bands: [
                EqualizerBand::new(32, -2.0),
                EqualizerBand::new(64, -1.0),
                EqualizerBand::new(125, 0.0),
                EqualizerBand::new(250, 2.0),
                EqualizerBand::new(500, 3.0),
                EqualizerBand::new(1000, 3.0),
                EqualizerBand::new(2000, 2.0),
                EqualizerBand::new(4000, 1.0),
                EqualizerBand::new(8000, -1.0),
                EqualizerBand::new(16000, -2.0),
            ],
        }
    }
}

/// Single band in the equalizer
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EqualizerBand {
    pub frequency: u32, // Hz
    pub gain: f32,      // dB (-12.0 to +12.0)
}

impl EqualizerBand {
    /// Creates a new equalizer band
    pub fn new(frequency: u32, gain: f32) -> Self {
        Self { frequency, gain }
    }
}

impl Default for EqualizerBand {
    fn default() -> Self {
        Self {
            frequency: 1000,
            gain: 0.0,
        }
    }
}

/// Sleep timer with fade-out
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SleepTimer {
    pub duration: Duration,
    pub fade_duration: Duration,
    pub started_at: Timestamp,
    pub state: SleepTimerState,
}

impl SleepTimer {
    /// Creates a new sleep timer
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            fade_duration: Duration::from_seconds(10),
            started_at: Timestamp::now(),
            state: SleepTimerState::Active,
        }
    }

    /// Creates a sleep timer with custom fade duration
    pub fn with_fade(duration: Duration, fade_duration: Duration) -> Self {
        Self {
            duration,
            fade_duration,
            started_at: Timestamp::now(),
            state: SleepTimerState::Active,
        }
    }

    /// Returns the remaining time in milliseconds
    pub fn remaining_millis(&self) -> i64 {
        let elapsed = Timestamp::now().as_millis() - self.started_at.as_millis();
        self.duration.as_millis() as i64 - elapsed
    }

    /// Returns true if the timer has expired
    pub fn is_expired(&self) -> bool {
        self.remaining_millis() <= 0
    }

    /// Returns true if the timer is in fade-out phase
    pub fn is_fading(&self) -> bool {
        let remaining = self.remaining_millis();
        remaining > 0 && remaining <= self.fade_duration.as_millis() as i64
    }
}

/// Sleep timer state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SleepTimerState {
    Active,
    Paused,
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_state_new() {
        let book_id = BookId::new();
        let state = PlaybackState::new(book_id);

        assert_eq!(state.book_id, book_id);
        assert_eq!(state.position.as_millis(), 0);
        assert!(!state.is_playing);
        assert_eq!(state.volume, 100);
    }

    #[test]
    fn test_playback_state_play_pause() {
        let mut state = PlaybackState::new(BookId::new());

        assert!(!state.is_playing);
        state.play();
        assert!(state.is_playing);
        state.pause();
        assert!(!state.is_playing);
    }

    #[test]
    fn test_playback_state_set_position() {
        let mut state = PlaybackState::new(BookId::new());
        let position = Duration::from_seconds(123);

        state.set_position(position);
        assert_eq!(state.position, position);
    }

    #[test]
    fn test_playback_state_validation() {
        let state = PlaybackState::new(BookId::new());
        assert!(state.is_valid());
    }

    #[test]
    fn test_playback_state_invalid_volume() {
        let mut state = PlaybackState::new(BookId::new());
        state.volume = 150;
        assert!(!state.is_valid());
    }

    #[test]
    fn test_playback_speed_default() {
        let speed = PlaybackSpeed::default();
        assert_eq!(speed.value(), 1.0);
        assert!(speed.has_pitch_correction());
    }

    #[test]
    fn test_playback_speed_new_valid() {
        let speed = PlaybackSpeed::new(1.5).unwrap();
        assert_eq!(speed.value(), 1.5);
    }

    #[test]
    fn test_playback_speed_new_invalid_low() {
        let result = PlaybackSpeed::new(0.3);
        assert!(result.is_err());
    }

    #[test]
    fn test_playback_speed_new_invalid_high() {
        let result = PlaybackSpeed::new(3.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_playback_speed_pitch_correction() {
        let speed = PlaybackSpeed::default().with_pitch_correction(false);
        assert!(!speed.has_pitch_correction());
    }

    #[test]
    fn test_playback_speed_validation() {
        let speed = PlaybackSpeed::new(2.0).unwrap();
        assert!(speed.is_valid());

        let invalid_speed = PlaybackSpeed::new_unchecked(5.0);
        assert!(!invalid_speed.is_valid());
    }

    #[test]
    fn test_equalizer_flat() {
        let eq = EqualizerPreset::flat();
        assert_eq!(eq.name, "Flat");
        assert_eq!(eq.bands.len(), 10);
        assert!(eq.bands.iter().all(|b| b.gain == 0.0));
    }

    #[test]
    fn test_equalizer_bass_boost() {
        let eq = EqualizerPreset::bass_boost();
        assert_eq!(eq.name, "Bass Boost");
        assert!(eq.bands[0].gain > 0.0);
        assert!(eq.bands[1].gain > 0.0);
    }

    #[test]
    fn test_equalizer_voice_boost() {
        let eq = EqualizerPreset::voice_boost();
        assert_eq!(eq.name, "Voice Boost");
        assert!(eq.bands[4].gain > 0.0); // 500 Hz
        assert!(eq.bands[5].gain > 0.0); // 1000 Hz
    }

    #[test]
    fn test_equalizer_band_default() {
        let band = EqualizerBand::default();
        assert_eq!(band.frequency, 1000);
        assert_eq!(band.gain, 0.0);
    }

    #[test]
    fn test_sleep_timer_new() {
        let timer = SleepTimer::new(Duration::from_seconds(1800));
        assert_eq!(timer.duration.as_seconds(), 1800);
        assert_eq!(timer.fade_duration.as_seconds(), 10);
        assert_eq!(timer.state, SleepTimerState::Active);
    }

    #[test]
    fn test_sleep_timer_with_fade() {
        let timer = SleepTimer::with_fade(
            Duration::from_seconds(1800),
            Duration::from_seconds(30),
        );
        assert_eq!(timer.fade_duration.as_seconds(), 30);
    }

    #[test]
    fn test_sleep_timer_remaining() {
        let timer = SleepTimer::new(Duration::from_seconds(60));
        let remaining = timer.remaining_millis();
        assert!(remaining > 0);
        assert!(remaining <= 60000);
    }

    #[test]
    fn test_sleep_timer_not_expired() {
        let timer = SleepTimer::new(Duration::from_seconds(60));
        assert!(!timer.is_expired());
    }

    #[test]
    fn test_sleep_timer_expired() {
        let timer = SleepTimer::new(Duration::from_millis(1));
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(timer.is_expired());
    }

    #[test]
    fn test_sleep_timer_fading() {
        let fade_duration = Duration::from_seconds(30);
        let mut timer = SleepTimer {
            duration: Duration::from_seconds(60),
            fade_duration,  // Just the variable, no Some()
            started_at: Timestamp::now(),
            state: SleepTimerState::Active,
        };

        // Not fading at start
        assert!(!timer.is_fading());

        // Simulate time passing to just before fade starts
        timer.started_at = Timestamp::from_millis(
            Timestamp::now().as_millis() - Duration::from_seconds(30).as_millis() as i64
        );

        assert!(timer.is_fading());

        // Simulate more time
        timer.started_at = Timestamp::from_millis(
            Timestamp::now().as_millis() - Duration::from_seconds(45).as_millis() as i64
        );
        assert!(timer.is_fading());
    }
}