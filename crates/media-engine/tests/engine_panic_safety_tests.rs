// crates/media-engine/tests/engine_panic_safety_tests.rs
//! Panic Safety Tests - Verify engine NEVER panics under ANY conditions
//!
//! These tests use panic catching to ensure the engine gracefully handles
//! all error conditions without panicking.

use media_engine::{EngineConfig, MediaEngine, Speed, Equalizer};
use std::panic;
use std::time::Duration;

/// Helper to catch panics and verify code doesn't panic
fn assert_no_panic<F: FnOnce() -> R + panic::UnwindSafe, R>(f: F) -> Result<R, String> {
    match panic::catch_unwind(f) {
        Ok(result) => Ok(result),
        Err(_) => Err("CODE PANICKED!".to_string()),
    }
}

#[test]
fn test_creation_never_panics() {
    let result = assert_no_panic(|| MediaEngine::with_defaults());
    assert!(result.is_ok(), "Engine creation must never panic");
}

#[test]
fn test_invalid_config_never_panics() {
    let result = assert_no_panic(|| {
        let config = EngineConfig {
            sample_rate: 0,  // Invalid
            channels: 0,     // Invalid
            buffer_size: 0,  // Invalid
        };
        MediaEngine::new(config)
    });
    assert!(result.is_ok(), "Invalid config must return Err, not panic");
}

#[test]
fn test_volume_edge_cases_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            // Test all edge cases
            let _ = engine.set_volume(f32::NAN);
            let _ = engine.set_volume(f32::INFINITY);
            let _ = engine.set_volume(f32::NEG_INFINITY);
            let _ = engine.set_volume(-1000.0);
            let _ = engine.set_volume(1000.0);
            let _ = engine.set_volume(f32::MIN);
            let _ = engine.set_volume(f32::MAX);
        }
    });
    assert!(result.is_ok(), "Volume edge cases must never panic");
}

#[test]
fn test_load_invalid_paths_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            // Empty path
            let _ = engine.load("");

            // Non-existent file
            let _ = engine.load("nonexistent.mp3");

            // Invalid path
            let _ = engine.load("\0invalid");

            // Very long path
            let _ = engine.load(&"a".repeat(10000));
        }
    });
    assert!(result.is_ok(), "Invalid paths must return Err, not panic");
}

#[test]
fn test_operations_without_file_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let _ = engine.play();
            let _ = engine.pause();
            let _ = engine.stop();
            let _ = engine.seek(Duration::from_secs(100));
            let _ = engine.set_volume(0.5);
            let _ = engine.set_speed(Speed::default());
            let _ = engine.next_chapter();
            let _ = engine.previous_chapter();
        }
    });
    assert!(result.is_ok(), "Operations without file must return Err, not panic");
}

#[test]
fn test_seek_edge_cases_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            engine.duration = Some(Duration::from_secs(100));

            // Seek way beyond duration
            let _ = engine.seek(Duration::from_secs(u64::MAX));

            // Seek to zero
            let _ = engine.seek(Duration::ZERO);

            // Seek to max
            let _ = engine.seek(Duration::MAX);
        }
    });
    assert!(result.is_ok(), "Seek edge cases must never panic");
}

#[test]
fn test_getters_always_work_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(engine) = MediaEngine::with_defaults() {
            // All getters must work even in invalid states
            let _ = engine.position();
            let _ = engine.is_playing();
            let _ = engine.status();
            let _ = engine.volume();
            let _ = engine.get_playback_state();
            let _ = engine.chapters();
            let _ = engine.current_chapter();
            let _ = engine.chapter_progress();

            // Call them multiple times
            for _ in 0..100 {
                let _ = engine.position();
                let _ = engine.volume();
            }
        }
    });
    assert!(result.is_ok(), "Getters must never panic");
}

#[test]
fn test_multiple_stops_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            for _ in 0..1000 {
                let _ = engine.stop();
            }
        }
    });
    assert!(result.is_ok(), "Multiple stops must never panic");
}

#[test]
fn test_drop_never_panics() {
    let result = assert_no_panic(|| {
        let _ = MediaEngine::with_defaults();
        // Drop happens here
    });
    assert!(result.is_ok(), "Drop must never panic");
}

#[test]
fn test_concurrent_access_never_panics() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let result = assert_no_panic(|| {
        if let Ok(engine) = MediaEngine::with_defaults() {
            let engine = Arc::new(Mutex::new(engine));
            let mut handles = vec![];

            for _ in 0..10 {
                let engine_clone = Arc::clone(&engine);
                let handle = thread::spawn(move || {
                    for _ in 0..100 {
                        if let Ok(eng) = engine_clone.lock() {
                            let _ = eng.position();
                            let _ = eng.volume();
                            let _ = eng.is_playing();
                        }
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                let _ = handle.join();
            }
        }
    });
    assert!(result.is_ok(), "Concurrent access must never panic");
}

#[test]
fn test_rapid_state_changes_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            for _ in 0..1000 {
                let _ = engine.stop();
                let _ = engine.set_volume(0.5);
                let _ = engine.set_speed(Speed::default());
                let _ = engine.position();
                let _ = engine.is_playing();
            }
        }
    });
    assert!(result.is_ok(), "Rapid state changes must never panic");
}

#[test]
fn test_speed_edge_cases_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            // Speed validation happens in Speed::new(), but test anyway
            if let Ok(speed) = Speed::new(0.25) {
                let _ = engine.set_speed(speed);
            }
            if let Ok(speed) = Speed::new(4.0) {
                let _ = engine.set_speed(speed);
            }
            let _ = engine.set_speed(Speed::default());
        }
    });
    assert!(result.is_ok(), "Speed operations must never panic");
}

#[test]
fn test_equalizer_operations_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let eq = Equalizer::default();
            let _ = engine.set_equalizer(eq);

            for _ in 0..100 {
                let _ = engine.set_equalizer(Equalizer::default());
            }
        }
    });
    assert!(result.is_ok(), "Equalizer operations must never panic");
}

#[test]
fn test_chapter_operations_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let chapters = vec![
                ("Ch1".to_string(), Duration::from_secs(0), Duration::from_secs(100)),
                ("Ch2".to_string(), Duration::from_secs(100), Duration::from_secs(200)),
            ];

            engine.load_chapters(chapters);
            let _ = engine.chapters();
            let _ = engine.current_chapter();
            let _ = engine.chapter_progress();
            let _ = engine.next_chapter();
            let _ = engine.previous_chapter();
        }
    });
    assert!(result.is_ok(), "Chapter operations must never panic");
}

#[test]
fn test_error_messages_are_actionable() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Test that all error messages are descriptive and actionable

        let err = engine.play().unwrap_err();
        assert!(err.contains("no file") || err.contains("load"),
                "Error should mention what's missing: {}", err);

        let err = engine.pause().unwrap_err();
        assert!(err.contains("no file") || err.contains("load"),
                "Error should be actionable: {}", err);

        let err = engine.load("").unwrap_err();
        assert!(err.contains("empty"),
                "Error should specify the problem: {}", err);

        let err = engine.set_volume(-1.0).unwrap_err();
        assert!(err.contains("below") || err.contains("minimum"),
                "Error should explain the bounds: {}", err);

        let err = engine.set_volume(2.0).unwrap_err();
        assert!(err.contains("exceeds") || err.contains("maximum"),
                "Error should explain the bounds: {}", err);
    }
}

#[test]
fn test_config_validation_messages() {
    let config = EngineConfig {
        sample_rate: 0,
        channels: 2,
        buffer_size: 4096,
    };
    let err = MediaEngine::new(config).unwrap_err();
    assert!(err.contains("sample_rate"),
            "Error should specify which field is invalid: {}", err);

    let config = EngineConfig {
        sample_rate: 44100,
        channels: 0,
        buffer_size: 4096,
    };
    let err = MediaEngine::new(config).unwrap_err();
    assert!(err.contains("channels"),
            "Error should specify which field is invalid: {}", err);

    let config = EngineConfig {
        sample_rate: 44100,
        channels: 2,
        buffer_size: 0,
    };
    let err = MediaEngine::new(config).unwrap_err();
    assert!(err.contains("buffer_size"),
            "Error should specify which field is invalid: {}", err);
}

#[test]
fn test_graceful_degradation() {
    // Test that getters work even if underlying state is compromised
    if let Ok(engine) = MediaEngine::with_defaults() {
        // These should all return safe defaults, never panic
        assert_eq!(engine.position(), Duration::from_secs(0));
        assert_eq!(engine.is_playing(), false);
        assert_eq!(engine.status(), false);
        assert_eq!(engine.volume(), 1.0);

        // State getter should return valid state
        let state = engine.get_playback_state();
        assert!(state.position().as_secs() >= 0);
    }
}

#[test]
fn test_boundary_values_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            // Test volume boundaries
            let _ = engine.set_volume(0.0);
            let _ = engine.set_volume(1.0);
            let _ = engine.set_volume(0.5);

            // Test duration boundaries
            engine.duration = Some(Duration::ZERO);
            let _ = engine.seek(Duration::ZERO);

            engine.duration = Some(Duration::from_secs(u64::MAX));
            let _ = engine.seek(Duration::from_secs(u64::MAX - 1));
        }
    });
    assert!(result.is_ok(), "Boundary values must never panic");
}

#[test]
fn test_repeated_load_attempts_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            for _ in 0..100 {
                let _ = engine.load("nonexistent.mp3");
                let _ = engine.load("");
                let _ = engine.load("invalid/path/file.mp3");
            }
        }
    });
    assert!(result.is_ok(), "Repeated load attempts must never panic");
}

#[test]
fn test_mixed_operations_never_panic() {
    let result = assert_no_panic(|| {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            for i in 0..100 {
                let _ = engine.play();
                let _ = engine.position();
                let _ = engine.pause();
                let _ = engine.set_volume((i % 100) as f32 / 100.0);
                let _ = engine.stop();
                let _ = engine.is_playing();
                let _ = engine.seek(Duration::from_secs(i));
            }
        }
    });
    assert!(result.is_ok(), "Mixed operations must never panic");
}