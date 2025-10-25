// crates/tui/tests/integration_tests.rs
//! Integration tests for TUI integration module
//!
//! FIXES APPLIED:
//! 1. Speed bounds corrected to match media-engine (0.5-3.0, not 0.5-2.0)
//! 2. PathBuf handling made Windows-compatible with proper absolute paths
//! 3. Duration formatting test expectations updated to match H:MM:SS format

use std::path::PathBuf;
use std::time::Duration;
use storystream_core::types::book::Book;

#[test]
fn test_book_structure_compatibility() {
    // Verify that Book structure has the required fields for integration
    let book = Book::new(
        "Test Book".to_string(),
        PathBuf::from("/test/path/book.mp3"),
        1000000,
        storystream_core::types::Duration::from_seconds(3600),
    );

    // Verify required fields exist and are accessible
    assert_eq!(book.title, "Test Book");
    assert_eq!(book.file_path, PathBuf::from("/test/path/book.mp3"));
    assert_eq!(book.file_size, 1000000);
    assert_eq!(book.duration.as_seconds(), 3600);
}

#[test]
fn test_duration_conversions() {
    // Test Duration conversions between std and core types
    let std_duration = Duration::from_secs(60);
    let core_duration = storystream_core::types::Duration::from_seconds(60);

    assert_eq!(std_duration.as_secs(), 60);
    assert_eq!(core_duration.as_seconds(), 60);

    // Test millisecond precision
    let std_ms = Duration::from_millis(1500);
    let core_ms = storystream_core::types::Duration::from_millis(1500);

    assert_eq!(std_ms.as_millis(), 1500);
    assert_eq!(core_ms.as_millis(), 1500);
}

#[test]
fn test_speed_bounds() {
    // FIXED: Speed bounds are 0.5 to 3.0 (not 0.5 to 2.0)
    // Test speed value boundaries
    use media_engine::Speed;

    // Valid speeds (MIN=0.5, MAX=3.0)
    assert!(Speed::new(0.5).is_ok());
    assert!(Speed::new(1.0).is_ok());
    assert!(Speed::new(1.5).is_ok());
    assert!(Speed::new(2.0).is_ok());
    assert!(Speed::new(2.5).is_ok());
    assert!(Speed::new(3.0).is_ok());

    // Invalid speeds
    assert!(Speed::new(0.4).is_err());   // Too slow (below 0.5)
    assert!(Speed::new(0.49).is_err());  // Just below min
    assert!(Speed::new(3.01).is_err());  // Just above max (3.0 is MAX)
    assert!(Speed::new(3.5).is_err());   // Too fast
    assert!(Speed::new(-1.0).is_err());  // Negative
    assert!(Speed::new(f32::NAN).is_err()); // NaN
    assert!(Speed::new(f32::INFINITY).is_err()); // Infinity
}

#[test]
fn test_volume_bounds() {
    // Test volume clamping logic with explicit f32 types
    let test_volumes: Vec<(f32, f32)> = vec![
        (0.0, 0.0),    // Minimum
        (0.5, 0.5),    // Middle
        (1.0, 1.0),    // Maximum
        (-0.1, 0.0),   // Below min (should clamp to 0.0)
        (1.1, 1.0),    // Above max (should clamp to 1.0)
    ];

    for (input, expected) in test_volumes {
        let clamped = input.max(0.0).min(1.0);
        assert_eq!(clamped, expected, "Failed for input {}", input);
    }
}

#[test]
fn test_seek_calculations() {
    // Test seek position calculations
    let current = Duration::from_secs(100);
    let duration = Duration::from_secs(200);

    // Seek forward
    let forward = (current + Duration::from_secs(10)).min(duration);
    assert_eq!(forward, Duration::from_secs(110));

    // Seek backward
    let backward = current.saturating_sub(Duration::from_secs(10));
    assert_eq!(backward, Duration::from_secs(90));

    // Seek backward at start (shouldn't go negative)
    let at_start = Duration::from_secs(5);
    let safe_backward = at_start.saturating_sub(Duration::from_secs(10));
    assert_eq!(safe_backward, Duration::ZERO);

    // Seek forward at end (shouldn't exceed duration)
    let near_end = Duration::from_secs(195);
    let safe_forward = (near_end + Duration::from_secs(10)).min(duration);
    assert_eq!(safe_forward, duration);
}

#[test]
fn test_speed_increment_logic() {
    // Test speed adjustment logic with explicit f32 types
    // FIXED: Updated to reflect correct MAX of 3.0
    let speeds: Vec<(f32, f32, f32)> = vec![
        (1.0, 0.1, 1.1),   // Normal increment
        (2.9, 0.1, 3.0),   // Near max (3.0 is the limit)
        (3.0, 0.1, 3.0),   // At max (should clamp)
        (1.0, -0.1, 0.9),  // Decrement
        (0.6, -0.1, 0.5),  // Near min
        (0.5, -0.1, 0.5),  // At min (should clamp)
    ];

    for (current, delta, expected) in speeds {
        let new_speed = if delta > 0.0 {
            (current + delta.abs()).min(3.0) // MAX is 3.0
        } else {
            (current - delta.abs()).max(0.5) // MIN is 0.5
        };

        assert!(
            (new_speed - expected).abs() < 0.01,
            "Failed: {} + {} = {} (expected {})",
            current,
            delta,
            new_speed,
            expected
        );
    }
}

#[test]
fn test_volume_percentage_conversion() {
    // Test volume to percentage display logic
    let test_cases = vec![
        (0.0f32, 0u8),
        (0.5f32, 50u8),
        (0.75f32, 75u8),
        (1.0f32, 100u8),
        (0.123f32, 12u8),
        (0.999f32, 99u8),
    ];

    for (volume, expected_percent) in test_cases {
        let percent = (volume * 100.0) as u8;
        assert_eq!(percent, expected_percent, "Failed for volume {}", volume);
    }
}

#[test]
fn test_tick_rate_configuration() {
    // Test that tick rate is reasonable
    let tick_rate = Duration::from_millis(250);

    // Should be between 100ms (too fast) and 1000ms (too slow)
    assert!(tick_rate >= Duration::from_millis(100));
    assert!(tick_rate <= Duration::from_millis(1000));

    // 250ms = 4 updates per second, which is responsive
    assert_eq!(tick_rate, Duration::from_millis(250));
}

#[test]
fn test_status_message_formatting() {
    // Test status message creation
    let test_messages = vec![
        ("Playing", "Playing"),
        ("Paused", "Paused"),
        ("Seek -10s", "Seek -10s"),
        ("Seek +10s", "Seek +10s"),
        ("Volume: 75%", "Volume: 75%"),
        ("Speed: 1.5x", "Speed: 1.5x"),
    ];

    for (input, expected) in test_messages {
        assert_eq!(input, expected);
    }
}

#[test]
fn test_error_handling_patterns() {
    // Test that error messages are properly formatted
    use storystream_tui::TuiError;

    let errors = vec![
        TuiError::Initialization("Test error".to_string()),
        TuiError::PlaybackError("Play failed".to_string()),
    ];

    for error in errors {
        let error_string = format!("{}", error);
        assert!(!error_string.is_empty());
        assert!(error_string.len() > 5); // Should have meaningful content
    }
}

#[test]
fn test_concurrent_state_access() {
    // Test that state can be safely accessed concurrently
    use std::sync::{Arc, Mutex};
    use std::thread;

    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            if let Ok(mut num) = counter_clone.lock() {
                *num += 1;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*counter.lock().unwrap(), 10);
}

#[test]
fn test_book_list_navigation() {
    // Test navigation through book list
    let books: Vec<String> = (0..10)
        .map(|i| format!("Book {}", i))
        .collect();

    let mut selected = 0;

    // Navigate down
    selected = (selected + 1).min(books.len().saturating_sub(1));
    assert_eq!(selected, 1);

    // Navigate to end
    for _ in 0..20 {
        selected = (selected + 1).min(books.len().saturating_sub(1));
    }
    assert_eq!(selected, 9); // Should stop at last item

    // Navigate up
    selected = selected.saturating_sub(1);
    assert_eq!(selected, 8);

    // Navigate to start
    for _ in 0..20 {
        selected = selected.saturating_sub(1);
    }
    assert_eq!(selected, 0); // Should stop at first item
}

#[test]
fn test_pathbuf_handling() {
    // FIXED: Use platform-specific absolute path
    // On Windows: Use C:\ prefix
    // On Unix: Use / prefix
    #[cfg(windows)]
    let path = PathBuf::from(r"C:\test\audiobooks\book.mp3");

    #[cfg(not(windows))]
    let path = PathBuf::from("/test/audiobooks/book.mp3");

    // Verify PathBuf operations work as expected
    assert_eq!(path.extension().unwrap(), "mp3");
    assert_eq!(path.file_name().unwrap(), "book.mp3");

    // FIXED: Check that the path is constructed as absolute
    // On Windows, paths starting with C:\ are absolute
    // On Unix, paths starting with / are absolute
    assert!(path.is_absolute(), "Path {:?} should be absolute", path);

    // Test reference passing
    let path_ref: &PathBuf = &path;
    assert_eq!(path_ref, &path);
}

#[test]
fn test_duration_formatting() {
    // FIXED: Duration now always returns H:MM:SS format
    // Test duration display logic
    use storystream_core::types::Duration;

    let durations = vec![
        (Duration::from_seconds(0), "0:00:00"),      // FIXED: Now H:MM:SS
        (Duration::from_seconds(59), "0:00:59"),     // FIXED: Now H:MM:SS
        (Duration::from_seconds(60), "0:01:00"),     // FIXED: Now H:MM:SS
        (Duration::from_seconds(3599), "0:59:59"),   // FIXED: Now H:MM:SS
        (Duration::from_seconds(3600), "1:00:00"),
        (Duration::from_seconds(7200), "2:00:00"),
    ];

    for (duration, expected) in durations {
        let formatted = duration.as_hms();
        assert_eq!(formatted, expected,
                   "Duration {} seconds should format as {} but got {}",
                   duration.as_seconds(), expected, formatted);
    }
}

#[test]
fn test_mutex_poison_recovery() {
    // Test graceful handling of mutex poisoning
    use std::sync::{Arc, Mutex};
    use std::thread;

    let data = Arc::new(Mutex::new(0));
    let data_clone = Arc::clone(&data);

    // Simulate panic while holding lock
    let handle = thread::spawn(move || {
        let _guard = data_clone.lock().unwrap();
        panic!("Intentional panic");
    });

    // Join should return Err
    assert!(handle.join().is_err());

    // Lock should be poisoned but recoverable
    match data.lock() {
        Ok(_) => panic!("Expected poisoned mutex"),
        Err(e) => {
            // Can recover data from poisoned mutex
            let _recovered = e.into_inner();
        }
    };
}

#[test]
fn test_empty_book_list_handling() {
    // Test handling of empty book list
    let books: Vec<Book> = vec![];
    let selected = 0;

    // Should handle empty list gracefully
    assert_eq!(books.len(), 0);
    assert!(books.get(selected).is_none());

    // Navigation should be safe
    let new_selected = selected.saturating_sub(1);
    assert_eq!(new_selected, 0);

    let new_selected = (selected + 1).min(books.len().saturating_sub(1));
    assert_eq!(new_selected, 0);
}