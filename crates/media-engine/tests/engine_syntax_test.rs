// crates/media-engine/tests/engine_syntax_test.rs

//! Comprehensive tests for MediaEngine syntax and compilation validation
//! This test file ensures all syntax errors are resolved and the code compiles correctly.

use media_engine::{EngineConfig, MediaEngine, Speed};
use std::time::Duration;

#[test]
fn test_engine_compiles_with_defaults() {
    let result = MediaEngine::with_defaults();
    assert!(result.is_ok(), "Engine should compile and create successfully");
}

#[test]
fn test_engine_compiles_with_custom_config() {
    let config = EngineConfig {
        sample_rate: 48000,
        channels: 2,
        buffer_size: 8192,
    };
    let result = MediaEngine::new(config);
    assert!(result.is_ok(), "Engine should compile with custom config");
}

#[test]
fn test_all_methods_compile() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Test that all methods compile without syntax errors
        let _ = engine.position();
        let _ = engine.is_playing();
        let _ = engine.status();
        let _ = engine.volume();
        let _ = engine.get_playback_state();
        let _ = engine.chapters();
        let _ = engine.current_chapter();
        let _ = engine.chapter_progress();

        // These will return errors but should compile
        let _ = engine.play();
        let _ = engine.pause();
        let _ = engine.stop();
        let _ = engine.seek(Duration::from_secs(10));
        let _ = engine.set_volume(0.5);
        let _ = engine.set_speed(Speed::default());
        let _ = engine.next_chapter();
        let _ = engine.previous_chapter();
    }
}

#[test]
fn test_engine_state_methods() {
    if let Ok(engine) = MediaEngine::with_defaults() {
        // Verify initial state
        assert!(!engine.is_playing(), "Engine should start not playing");
        assert_eq!(engine.position(), Duration::from_secs(0), "Position should start at 0");
        assert_eq!(engine.volume(), 1.0, "Volume should start at 1.0");

        let state = engine.get_playback_state();
        assert!(state.position().as_secs() == 0, "State position should be 0");
    }
}

#[test]
fn test_volume_validation() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Valid volumes
        assert!(engine.set_volume(0.0).is_ok());
        assert!(engine.set_volume(0.5).is_ok());
        assert!(engine.set_volume(1.0).is_ok());

        // Invalid volumes
        assert!(engine.set_volume(-0.1).is_err(), "Negative volume should error");
        assert!(engine.set_volume(1.1).is_err(), "Volume > 1.0 should error");
        assert!(engine.set_volume(2.0).is_err(), "Volume > 1.0 should error");
    }
}

#[test]
fn test_operations_without_loaded_file() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // These should all return errors when no file is loaded
        assert!(engine.play().is_err(), "Play without file should error");
        assert!(engine.pause().is_err(), "Pause without file should error");
        assert!(engine.seek(Duration::from_secs(10)).is_err(), "Seek without file should error");

        // Stop should always succeed
        assert!(engine.stop().is_ok(), "Stop should always succeed");
    }
}

#[test]
fn test_load_invalid_file() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Try to load a non-existent file
        let result = engine.load("this_file_does_not_exist.mp3");
        assert!(result.is_err(), "Loading non-existent file should error");

        // Try to load a file with invalid extension
        let result = engine.load("invalid.xyz");
        assert!(result.is_err(), "Loading invalid file type should error");
    }
}

#[test]
fn test_chapter_methods_compile() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Test chapter methods compile (even if not fully implemented)
        let chapters = vec![
            ("Chapter 1".to_string(), Duration::from_secs(0), Duration::from_secs(100)),
            ("Chapter 2".to_string(), Duration::from_secs(100), Duration::from_secs(200)),
        ];

        engine.load_chapters(chapters);

        let _ = engine.chapters();
        let _ = engine.current_chapter();
        let _ = engine.chapter_progress();

        // These should return "not implemented" errors but should compile
        assert!(engine.next_chapter().is_err());
        assert!(engine.previous_chapter().is_err());
    }
}

#[test]
fn test_multiple_stop_calls_safe() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Multiple stop calls should be safe
        assert!(engine.stop().is_ok());
        assert!(engine.stop().is_ok());
        assert!(engine.stop().is_ok());
    }
}

#[test]
fn test_engine_drop_is_safe() {
    // Engine should drop safely even without stopping
    let _ = MediaEngine::with_defaults();
    // If we get here, drop worked
}

#[test]
fn test_concurrent_access_safety() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    if let Ok(engine) = MediaEngine::with_defaults() {
        let engine = Arc::new(Mutex::new(engine));
        let mut handles = vec![];

        // Spawn multiple threads accessing the engine
        for _ in 0..10 {
            let engine_clone = Arc::clone(&engine);
            let handle = thread::spawn(move || {
                if let Ok(eng) = engine_clone.lock() {
                    let _ = eng.position();
                    let _ = eng.is_playing();
                    let _ = eng.volume();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            assert!(handle.join().is_ok(), "Thread should complete successfully");
        }
    }
}

#[test]
fn test_speed_operations() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Test speed setting compiles
        let speed = Speed::default();
        assert!(engine.set_speed(speed).is_ok());

        // Test various speed values
        if let Ok(speed) = Speed::new(1.5) {
            assert!(engine.set_speed(speed).is_ok());
        }

        if let Ok(speed) = Speed::new(0.5) {
            assert!(engine.set_speed(speed).is_ok());
        }
    }
}

#[test]
fn test_equalizer_operations() {
    use media_engine::Equalizer;

    if let Ok(mut engine) = MediaEngine::with_defaults() {
        let eq = Equalizer::default();
        assert!(engine.set_equalizer(eq).is_ok(), "Set equalizer should succeed");
    }
}

#[test]
fn test_position_tracking() {
    if let Ok(engine) = MediaEngine::with_defaults() {
        // Position should start at 0
        assert_eq!(engine.position(), Duration::from_secs(0));

        // Multiple calls should be consistent
        assert_eq!(engine.position(), engine.position());
    }
}

#[test]
fn test_status_consistency() {
    if let Ok(engine) = MediaEngine::with_defaults() {
        // Status methods should be consistent
        assert_eq!(engine.is_playing(), engine.status());

        // Both should report not playing initially
        assert!(!engine.is_playing());
        assert!(!engine.status());
    }
}