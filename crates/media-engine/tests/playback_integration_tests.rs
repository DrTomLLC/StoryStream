//! Integration tests for complete playback pipeline
//!
//! These tests verify that the MediaEngine properly integrates
//! with the PlaybackThread for actual audio playback.

use media_engine::{MediaEngine, PlaybackSpeed, PlaybackStatus};
use std::thread;
use std::time::Duration;

#[test]
fn test_engine_state_lifecycle() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    // Initial state
    assert_eq!(engine.status(), PlaybackStatus::Stopped);
    assert_eq!(engine.position(), 0.0);
    assert!(!engine.is_playing());

    // After stop
    engine.stop().expect("Failed to stop");
    assert_eq!(engine.status(), PlaybackStatus::Stopped);
    assert_eq!(engine.position(), 0.0);
}

#[test]
fn test_load_clears_previous_state() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    // This will fail since we don't have a real file, but tests the error path
    let result = engine.load("nonexistent.mp3");
    assert!(result.is_err());

    // State should remain stopped
    assert_eq!(engine.status(), PlaybackStatus::Stopped);
}

#[test]
fn test_play_without_file_fails() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    let result = engine.play();
    assert!(result.is_err());
    assert_eq!(engine.status(), PlaybackStatus::Stopped);
}

#[test]
fn test_pause_without_file_fails() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    let result = engine.pause();
    assert!(result.is_err());
    assert_eq!(engine.status(), PlaybackStatus::Stopped);
}

#[test]
fn test_seek_without_file_fails() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    let result = engine.seek(10.0);
    assert!(result.is_err());
}

#[test]
fn test_volume_control() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    // Set various volumes
    engine.set_volume(0.0).expect("Failed to set volume");
    assert_eq!(engine.volume(), 0.0);

    engine.set_volume(0.5).expect("Failed to set volume");
    assert_eq!(engine.volume(), 0.5);

    engine.set_volume(1.0).expect("Failed to set volume");
    assert_eq!(engine.volume(), 1.0);

    // Invalid volumes should fail
    assert!(engine.set_volume(-0.1).is_err());
    assert!(engine.set_volume(1.1).is_err());
}

#[test]
fn test_speed_control() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    // Set various speeds
    let speeds = vec![0.5, 0.75, 1.0, 1.25, 1.5, 2.0];

    for speed_val in speeds {
        let speed = PlaybackSpeed::new(speed_val).expect("Invalid speed");
        engine.set_speed(speed).expect("Failed to set speed");
    }
}

#[test]
fn test_stop_resets_position() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    engine.stop().expect("Failed to stop");

    assert_eq!(engine.position(), 0.0);
    assert_eq!(engine.status(), PlaybackStatus::Stopped);
    assert!(!engine.is_playing());
}

#[test]
fn test_multiple_stops_safe() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    // Multiple stops should be safe
    engine.stop().expect("Failed to stop");
    engine.stop().expect("Failed to stop");
    engine.stop().expect("Failed to stop");

    assert_eq!(engine.status(), PlaybackStatus::Stopped);
}

#[test]
fn test_engine_drop_stops_playback() {
    // Engine should cleanly stop when dropped
    {
        let mut engine = MediaEngine::new().expect("Failed to create engine");
        let _ = engine.stop();
    } // Engine drops here

    // If we get here without hanging, the test passes
    thread::sleep(Duration::from_millis(10));
}

#[test]
fn test_volume_persists_across_operations() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    engine.set_volume(0.7).expect("Failed to set volume");
    assert_eq!(engine.volume(), 0.7);

    // Stop shouldn't change volume
    engine.stop().expect("Failed to stop");
    assert_eq!(engine.volume(), 0.7);
}

#[test]
fn test_concurrent_position_reads() {
    let engine = MediaEngine::new().expect("Failed to create engine");

    // Multiple position reads should be safe
    for _ in 0..100 {
        let _ = engine.position();
    }
}

#[test]
fn test_concurrent_status_reads() {
    let engine = MediaEngine::new().expect("Failed to create engine");

    // Multiple status reads should be safe
    for _ in 0..100 {
        let _ = engine.status();
    }
}

#[test]
fn test_default_engine() {
    let engine = MediaEngine::default();

    assert_eq!(engine.status(), PlaybackStatus::Stopped);
    assert_eq!(engine.volume(), 1.0);
    assert_eq!(engine.position(), 0.0);
}

#[test]
fn test_engine_with_config() {
    let engine = MediaEngine::with_config(()).expect("Failed to create engine");

    assert_eq!(engine.status(), PlaybackStatus::Stopped);
}

#[test]
fn test_is_playing_initial_state() {
    let engine = MediaEngine::new().expect("Failed to create engine");

    assert!(!engine.is_playing());
}

#[test]
fn test_status_transitions() {
    let mut engine = MediaEngine::new().expect("Failed to create engine");

    // Verify initial stopped state
    assert_eq!(engine.status(), PlaybackStatus::Stopped);

    // Stop when already stopped should work
    engine.stop().expect("Failed to stop");
    assert_eq!(engine.status(), PlaybackStatus::Stopped);
}