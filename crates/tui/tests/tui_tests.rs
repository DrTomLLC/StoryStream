// crates/tui/tests/tui_tests.rs
//! Integration tests for TUI

use storystream_tui::{App, AppState, PlaybackState, View};
use std::time::Duration;

#[test]
fn test_app_state_creation() {
    let state = AppState::new();
    assert_eq!(state.view, View::Library);
    assert!(!state.should_quit);
}

#[test]
fn test_app_state_view_switching() {
    let mut state = AppState::new();

    state.set_view(View::Player);
    assert_eq!(state.view, View::Player);

    state.set_view(View::Bookmarks);
    assert_eq!(state.view, View::Bookmarks);
}

#[test]
fn test_app_state_quit() {
    let mut state = AppState::new();
    assert!(!state.should_quit);

    state.quit();
    assert!(state.should_quit);
}

#[test]
fn test_app_state_status_messages() {
    let mut state = AppState::new();

    state.set_status("Test message");
    assert_eq!(state.status_message, Some("Test message".to_string()));

    state.clear_status();
    assert_eq!(state.status_message, None);
}

#[test]
fn test_app_state_selection_navigation() {
    let mut state = AppState::new();
    state.library_items_count = 10;

    assert_eq!(state.selected_item, 0);

    state.select_next();
    assert_eq!(state.selected_item, 1);

    state.select_next();
    assert_eq!(state.selected_item, 2);

    state.select_previous();
    assert_eq!(state.selected_item, 1);

    // Can't go below 0
    state.reset_selection();
    state.select_previous();
    assert_eq!(state.selected_item, 0);

    // Can't exceed count
    for _ in 0..20 {
        state.select_next();
    }
    assert_eq!(state.selected_item, 9);
}

#[test]
fn test_playback_state_default() {
    let state = PlaybackState::default();

    assert!(state.current_file.is_none());
    assert_eq!(state.position, Duration::ZERO);
    assert_eq!(state.duration, Duration::ZERO);
    assert!(!state.is_playing);
    assert_eq!(state.volume, 1.0);
    assert_eq!(state.speed, 1.0);
    assert_eq!(state.chapter, None);
}

#[test]
fn test_playback_state_progress() {
    let mut state = PlaybackState::default();
    state.duration = Duration::from_secs(100);

    state.position = Duration::from_secs(0);
    assert_eq!(state.progress(), 0.0);

    state.position = Duration::from_secs(50);
    assert_eq!(state.progress(), 0.5);

    state.position = Duration::from_secs(100);
    assert_eq!(state.progress(), 1.0);
}

#[test]
fn test_playback_state_remaining() {
    let mut state = PlaybackState::default();
    state.duration = Duration::from_secs(100);
    state.position = Duration::from_secs(30);

    assert_eq!(state.remaining(), Duration::from_secs(70));
}

#[test]
fn test_playback_state_time_formatting() {
    let mut state = PlaybackState::default();

    // Short duration
    state.position = Duration::from_secs(125); // 2:05
    state.duration = Duration::from_secs(300); // 5:00

    assert_eq!(state.format_position(), "02:05");
    assert_eq!(state.format_duration(), "05:00");

    // Long duration
    state.position = Duration::from_secs(3665); // 1:01:05
    state.duration = Duration::from_secs(7200); // 2:00:00

    assert_eq!(state.format_position(), "01:01:05");
    assert_eq!(state.format_duration(), "02:00:00");
}

#[test]
fn test_view_enum_variants() {
    assert_eq!(View::default(), View::Library);

    let views = [
        View::Library,
        View::Player,
        View::Bookmarks,
        View::Settings,
        View::Help,
    ];

    for view in views {
        let _ = view; // Just verify they all exist
    }
}

#[test]
fn test_app_creation() {
    let app = App::new();
    let _ = app;
}

#[test]
fn test_app_default() {
    let app = App::default();
    let _ = app;
}