// crates/tui/tests/tab_navigation_tests.rs
//! Integration tests for tab navigation and state preservation

use std::time::Duration;
use storystream_tui::{App, AppState, PlaybackState, View};

#[test]
fn test_basic_view_state_preservation() {
    let mut state = AppState::new();
    state.library_items_count = 10;

    // Navigate in Library view
    state.select_next();
    state.select_next();
    state.select_next();
    assert_eq!(state.selected_item, 3);

    // Switch to Player view
    state.set_view(View::Player);
    assert_eq!(state.selected_item, 0); // Fresh view starts at 0

    // Switch back to Library view
    state.set_view(View::Library);
    assert_eq!(state.selected_item, 3); // Should restore position 3
}

#[test]
fn test_multiple_view_state_preservation() {
    let mut state = AppState::new();
    state.library_items_count = 10;

    // Set up different positions in different views

    // Library: position 3
    state.set_view(View::Library);
    for _ in 0..3 {
        state.select_next();
    }
    assert_eq!(state.selected_item, 3);

    // Bookmarks: position 2
    state.set_view(View::Bookmarks);
    state.select_next();
    state.select_next();
    assert_eq!(state.selected_item, 2);

    // Settings: position 5
    state.set_view(View::Settings);
    for _ in 0..5 {
        state.select_next();
    }
    assert_eq!(state.selected_item, 5);

    // Verify all positions are preserved when returning
    state.set_view(View::Library);
    assert_eq!(state.selected_item, 3);

    state.set_view(View::Bookmarks);
    assert_eq!(state.selected_item, 2);

    state.set_view(View::Settings);
    assert_eq!(state.selected_item, 5);
}

#[test]
fn test_tab_cycling_preserves_all_states() {
    let mut app = App::new();
    app.state.library_items_count = 10;

    // Set different positions in multiple views
    // Library: position 4
    app.state.set_view(View::Library);
    for _ in 0..4 {
        app.state.select_next();
    }
    let library_pos = app.state.selected_item;
    assert_eq!(library_pos, 4);

    // Bookmarks: position 1
    app.state.set_view(View::Bookmarks);
    app.state.select_next();
    let bookmarks_pos = app.state.selected_item;
    assert_eq!(bookmarks_pos, 1);

    // Search: position 3
    app.state.set_view(View::Search);
    for _ in 0..3 {
        app.state.select_next();
    }
    let search_pos = app.state.selected_item;
    assert_eq!(search_pos, 3);

    // Now tab through all views and come back to Library
    // From Search: Search -> Playlists -> Statistics -> Settings -> Help -> Library (5 cycles)
    for _ in 0..5 {
        app.cycle_view();
    }
    assert_eq!(app.state.view, View::Library);
    assert_eq!(app.state.selected_item, library_pos);

    // Go to Bookmarks
    app.state.set_view(View::Bookmarks);
    assert_eq!(app.state.selected_item, bookmarks_pos);

    // Go to Search
    app.state.set_view(View::Search);
    assert_eq!(app.state.selected_item, search_pos);
}

#[test]
fn test_reverse_tab_cycling_preserves_states() {
    let mut app = App::new();
    app.state.library_items_count = 10;

    // Set position in Library
    app.state.set_view(View::Library);
    for _ in 0..5 {
        app.state.select_next();
    }
    assert_eq!(app.state.selected_item, 5);

    // Cycle forward to Settings
    while app.state.view != View::Settings {
        app.cycle_view();
    }

    // Set position in Settings
    app.state.select_next();
    app.state.select_next();
    assert_eq!(app.state.selected_item, 2);

    // Cycle backward to Library
    while app.state.view != View::Library {
        app.cycle_view_reverse();
    }

    // Library position should be preserved
    assert_eq!(app.state.selected_item, 5);
}

#[test]
fn test_selection_navigation_updates_view_state() {
    let mut state = AppState::new();
    state.library_items_count = 10;

    state.set_view(View::Library);

    // Navigate and ensure state is continuously updated
    state.select_next(); // 1
    state.set_view(View::Player);
    state.set_view(View::Library);
    assert_eq!(state.selected_item, 1);

    state.select_next(); // 2
    state.set_view(View::Bookmarks);
    state.set_view(View::Library);
    assert_eq!(state.selected_item, 2);

    state.select_previous(); // 1
    state.set_view(View::Settings);
    state.set_view(View::Library);
    assert_eq!(state.selected_item, 1);
}

#[test]
fn test_no_cursor_jump_on_rapid_tab_switches() {
    let mut app = App::new();
    app.state.library_items_count = 10;

    // Set a specific position in Library
    app.state.set_view(View::Library);
    for _ in 0..7 {
        app.state.select_next();
    }
    assert_eq!(app.state.selected_item, 7);

    // Rapidly switch tabs back and forth
    for _ in 0..5 {
        app.cycle_view(); // Away from Library
        app.cycle_view(); // Further away
        app.cycle_view(); // Even further
                          // Jump back to Library
        app.state.set_view(View::Library);
        // Position should still be 7
        assert_eq!(app.state.selected_item, 7);
    }
}

#[test]
fn test_mouse_click_tab_switch_preserves_state() {
    let mut app = App::new();
    app.state.library_items_count = 10;

    // Navigate in Library
    for _ in 0..6 {
        app.state.select_next();
    }
    assert_eq!(app.state.selected_item, 6);

    // Simulate mouse click tab switches
    app.state.set_view(View::Player);
    assert_eq!(app.state.selected_item, 0); // New view

    app.state.set_view(View::Bookmarks);
    assert_eq!(app.state.selected_item, 0); // New view

    app.state.set_view(View::Library);
    assert_eq!(app.state.selected_item, 6); // Restored!
}

#[test]
fn test_bounds_checking_with_state_preservation() {
    let mut state = AppState::new();
    state.library_items_count = 5;

    state.set_view(View::Library);

    // Go to max position
    for _ in 0..10 {
        state.select_next();
    }
    assert_eq!(state.selected_item, 4); // Max is 5-1=4

    // Switch views and back
    state.set_view(View::Player);
    state.set_view(View::Library);

    // Should still be at max
    assert_eq!(state.selected_item, 4);

    // Try to go past max
    state.select_next();
    assert_eq!(state.selected_item, 4); // Still at max
}

#[test]
fn test_reset_selection_only_resets_current_view() {
    let mut state = AppState::new();
    state.library_items_count = 10;

    // Set positions in multiple views
    state.set_view(View::Library);
    for _ in 0..5 {
        state.select_next();
    }

    state.set_view(View::Bookmarks);
    for _ in 0..3 {
        state.select_next();
    }

    // Reset selection in Bookmarks
    state.reset_selection();
    assert_eq!(state.selected_item, 0);

    // Library should still have its old position
    state.set_view(View::Library);
    assert_eq!(state.selected_item, 5);
}

#[test]
fn test_new_view_starts_at_zero() {
    let mut state = AppState::new();

    // First visit to Settings should start at 0
    state.set_view(View::Settings);
    assert_eq!(state.selected_item, 0);

    // First visit to Search should start at 0
    state.set_view(View::Search);
    assert_eq!(state.selected_item, 0);
}

#[test]
fn test_comprehensive_workflow() {
    let mut app = App::new();
    app.state.library_items_count = 10;

    // User workflow: Browse library, check bookmarks, search, back to library

    // 1. Browse library to position 8
    app.state.set_view(View::Library);
    for _ in 0..8 {
        app.state.select_next();
    }
    assert_eq!(app.state.selected_item, 8);

    // 2. Tab to Player
    app.cycle_view();
    assert_eq!(app.state.view, View::Player);

    // 3. Tab to Bookmarks and navigate
    app.cycle_view();
    assert_eq!(app.state.view, View::Bookmarks);
    app.state.select_next();
    app.state.select_next();
    assert_eq!(app.state.selected_item, 2);

    // 4. Tab to Search and navigate
    app.cycle_view();
    assert_eq!(app.state.view, View::Search);
    for _ in 0..4 {
        app.state.select_next();
    }
    assert_eq!(app.state.selected_item, 4);

    // 5. Jump back to Library with 'h' key (simulated)
    app.state.set_view(View::Library);

    // Library position should be exactly where we left it
    assert_eq!(app.state.selected_item, 8);

    // 6. Go back to Bookmarks
    app.state.set_view(View::Bookmarks);
    assert_eq!(app.state.selected_item, 2);

    // 7. Go back to Search
    app.state.set_view(View::Search);
    assert_eq!(app.state.selected_item, 4);
}

#[test]
fn test_app_state_equality() {
    let mut state1 = AppState::new();
    let mut state2 = AppState::new();

    // Both start the same
    assert_eq!(state1.view, state2.view);
    assert_eq!(state1.selected_item, state2.selected_item);

    // Navigate identically
    state1.select_next();
    state2.select_next();
    assert_eq!(state1.selected_item, state2.selected_item);

    // Switch views identically
    state1.set_view(View::Player);
    state2.set_view(View::Player);
    assert_eq!(state1.view, state2.view);
    assert_eq!(state1.selected_item, state2.selected_item);
}
