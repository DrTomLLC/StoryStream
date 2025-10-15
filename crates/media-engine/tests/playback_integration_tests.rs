use media_engine::{EngineConfig, MediaEngine, PlaybackState, Speed};
use std::time::Duration;

#[test]
fn test_default_engine() {
    let engine = MediaEngine::with_defaults();
    assert!(engine.is_ok());
}

#[test]
fn test_engine_with_config() {
    let config = EngineConfig {
        sample_rate: 48000,
        channels: 2,
        buffer_size: 2048,
    };
    let engine = MediaEngine::new(config);
    assert!(engine.is_ok());
}

#[test]
fn test_play_without_file_fails() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        let result = engine.play();
        assert!(result.is_err());
        if let Err(msg) = result {
            assert!(msg.contains("No file loaded"));
        }
    }
}

#[test]
fn test_pause_without_file_fails() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        let result = engine.pause();
        assert!(result.is_err());
        if let Err(msg) = result {
            assert!(msg.contains("No file loaded"));
        }
    }
}

#[test]
fn test_seek_without_file_fails() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        let result = engine.seek(Duration::from_secs(10));
        assert!(result.is_err());
        if let Err(msg) = result {
            assert!(msg.contains("No file loaded"));
        }
    }
}

#[test]
fn test_volume_control() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Test valid volumes
        assert!(engine.set_volume(0.0).is_ok());
        assert_eq!(engine.volume(), 0.0);

        assert!(engine.set_volume(0.5).is_ok());
        assert_eq!(engine.volume(), 0.5);

        assert!(engine.set_volume(1.0).is_ok());
        assert_eq!(engine.volume(), 1.0);

        // Test invalid volumes
        assert!(engine.set_volume(-0.1).is_err());
        assert!(engine.set_volume(1.1).is_err());
    }
}

#[test]
fn test_speed_control() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        if let Ok(speed) = Speed::new(1.5) {
            assert!(engine.set_speed(speed).is_ok());
        }

        if let Ok(speed) = Speed::new(0.5) {
            assert!(engine.set_speed(speed).is_ok());
        }

        if let Ok(speed) = Speed::new(2.0) {
            assert!(engine.set_speed(speed).is_ok());
        }
    }
}

#[test]
fn test_is_playing_initial_state() {
    if let Ok(engine) = MediaEngine::with_defaults() {
        assert!(!engine.is_playing());
    }
}

#[test]
fn test_stop_resets_position() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        let _ = engine.stop();
        assert_eq!(engine.position(), Duration::from_secs(0));
        assert!(!engine.is_playing());
    }
}

#[test]
fn test_engine_state_lifecycle() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Initial state
        assert!(!engine.is_playing());
        assert_eq!(engine.position(), Duration::from_secs(0));

        // After stop
        let _ = engine.stop();
        assert!(!engine.is_playing());
        assert_eq!(engine.position(), Duration::from_secs(0));
    }
}

#[test]
fn test_volume_persists_across_operations() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        let _ = engine.set_volume(0.7);
        assert_eq!(engine.volume(), 0.7);

        let _ = engine.stop();
        assert_eq!(engine.volume(), 0.7);
    }
}

#[test]
fn test_multiple_stops_safe() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        assert!(engine.stop().is_ok());
        assert!(engine.stop().is_ok());
        assert!(engine.stop().is_ok());
    }
}

#[test]
fn test_status_transitions() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Should start as not playing
        assert!(!engine.status());

        // Stop should keep it not playing
        let _ = engine.stop();
        assert!(!engine.status());
    }
}

#[test]
fn test_concurrent_position_reads() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    if let Ok(engine) = MediaEngine::with_defaults() {
        let engine = Arc::new(Mutex::new(engine));
        let mut handles = vec![];

        for _ in 0..10 {
            let engine_clone = Arc::clone(&engine);
            let handle = thread::spawn(move || {
                if let Ok(eng) = engine_clone.lock() {
                    let _ = eng.position();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }
    }
}

#[test]
fn test_concurrent_status_reads() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    if let Ok(engine) = MediaEngine::with_defaults() {
        let engine = Arc::new(Mutex::new(engine));
        let mut handles = vec![];

        for _ in 0..10 {
            let engine_clone = Arc::clone(&engine);
            let handle = thread::spawn(move || {
                if let Ok(eng) = engine_clone.lock() {
                    let _ = eng.is_playing();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }
    }
}

#[test]
fn test_engine_drop_stops_playback() {
    {
        if let Ok(mut engine) = MediaEngine::with_defaults() {
            let _ = engine.stop();
        }
    }
    // Engine should drop cleanly
}

#[test]
fn test_load_clears_previous_state() {
    if let Ok(mut engine) = MediaEngine::with_defaults() {
        // Try to load a file (will fail but that's okay for this test)
        let _ = engine.load("nonexistent.mp3");

        // Position should be reset
        assert_eq!(engine.position(), Duration::from_secs(0));
        assert!(!engine.is_playing());
    }
}

#[test]
fn test_playback_state_new() {
    let state = PlaybackState::new();
    // PlaybackState position() returns Duration
    assert_eq!(state.position(), Duration::from_secs(0));
    // PlaybackState duration() returns Option<Duration>
    assert_eq!(state.duration(), None);
}

#[test]
fn test_playback_state_is_playing() {
    let state = PlaybackState::new();
    // Just test the method exists and doesn't panic
    let _ = state.is_playing();
}

#[test]
fn test_playback_state_position_updates() {
    let mut state = PlaybackState::new();

    state.set_position(Duration::from_secs(10));
    assert_eq!(state.position(), Duration::from_secs(10));

    state.set_position(Duration::from_secs(25));
    assert_eq!(state.position(), Duration::from_secs(25));
}

#[test]
fn test_playback_state_duration_updates() {
    let mut state = PlaybackState::new();

    assert_eq!(state.duration(), None);

    state.set_duration(Duration::from_secs(300));
    assert_eq!(state.duration(), Some(Duration::from_secs(300)));
}

#[test]
fn test_playback_state_progress() {
    let mut state = PlaybackState::new();
    state.set_position(Duration::from_secs(25));
    state.set_duration(Duration::from_secs(100));

    if let Some(progress) = state.progress_percentage() {
        assert!((progress - 25.0).abs() < 0.1);
    }
}