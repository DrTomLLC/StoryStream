// crates/tui/src/state.rs - CORRECTED VERSION
//! Application state management

use std::collections::HashMap;
use std::time::Duration;

/// Available views
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum View {
    Library,
    Player,
    Bookmarks,
    Search,
    Playlists,
    Statistics,
    Settings,
    Help,
    Plugin,
}

impl Default for View {
    fn default() -> Self {
        Self::Library
    }
}

/// Playback state
#[derive(Debug, Clone)]
pub struct PlaybackState {
    /// Current file being played
    pub current_file: Option<String>,
    /// Current position in the audiobook
    pub position: Duration,
    /// Total duration
    pub duration: Duration,
    /// Whether playback is active
    pub is_playing: bool,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Playback speed (0.5 to 3.0)
    pub speed: f32,
    /// Current chapter (index, not tuple)
    pub chapter: Option<usize>,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            current_file: None,
            position: Duration::ZERO,
            duration: Duration::ZERO,
            is_playing: false,
            volume: 1.0,
            speed: 1.0,
            chapter: None,
        }
    }
}

impl PlaybackState {
    /// Returns playback progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.duration.as_secs() == 0 {
            return 0.0;
        }
        self.position.as_secs_f32() / self.duration.as_secs_f32()
    }

    /// Returns remaining time
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.position)
    }

    /// Formats position as MM:SS or HH:MM:SS
    pub fn format_position(&self) -> String {
        format_duration(self.position)
    }

    /// Formats duration as MM:SS or HH:MM:SS
    pub fn format_duration(&self) -> String {
        format_duration(self.duration)
    }
}

/// Application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current view
    pub view: View,
    /// Should quit
    pub should_quit: bool,
    /// Playback state
    pub playback: PlaybackState,
    /// Selected library item
    pub selected_item: usize,
    /// Library items count
    pub library_items_count: usize,
    /// Status message
    pub status_message: Option<String>,
    /// Search query
    pub search_query: String,
    /// Mouse position
    pub mouse_position: Option<(u16, u16)>,
    /// Theme type
    pub theme: crate::theme::ThemeType,
    /// Per-view selection states (preserves cursor position when switching views)
    view_selections: HashMap<View, usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: View::default(),
            should_quit: false,
            playback: PlaybackState::default(),
            selected_item: 0,
            library_items_count: 8, // Demo books
            status_message: None,
            search_query: String::new(),
            mouse_position: None,
            theme: crate::theme::ThemeType::default(),
            view_selections: HashMap::new(),
        }
    }
}

impl AppState {
    /// Creates a new application state
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the current view and preserves/restores selection state
    pub fn set_view(&mut self, view: View) {
        // Save current selection for current view
        self.save_view_selection();

        // Switch view
        self.view = view;

        // Restore selection for new view
        self.restore_view_selection();
    }

    /// Saves the current selection for the current view
    fn save_view_selection(&mut self) {
        self.view_selections.insert(self.view, self.selected_item);
    }

    /// Restores the selection for the current view (or defaults to 0)
    fn restore_view_selection(&mut self) {
        self.selected_item = *self.view_selections.get(&self.view).unwrap_or(&0);
    }

    /// Requests quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Sets a status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Clears the status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Sets the search query
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    /// Clears the search query
    pub fn clear_search_query(&mut self) {
        self.search_query.clear();
    }

    /// Selects the next item in the current view
    pub fn select_next(&mut self) {
        let max_item = self.get_max_items_for_view().saturating_sub(1);
        if self.selected_item < max_item {
            self.selected_item += 1;
            self.save_view_selection(); // Save immediately
        }
    }

    /// Selects the previous item in the current view
    pub fn select_previous(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
            self.save_view_selection(); // Save immediately
        }
    }

    /// Resets selection to the first item
    pub fn reset_selection(&mut self) {
        self.selected_item = 0;
        self.save_view_selection(); // Save immediately
    }

    /// Gets the maximum number of items for the current view
    fn get_max_items_for_view(&self) -> usize {
        match self.view {
            View::Library => self.library_items_count,
            View::Bookmarks => 10, // Example count
            View::Search => 15,    // Example count
            View::Playlists => 5,  // Example count
            View::Settings => 10,  // Example count
            View::Statistics => 5, // Example count
            _ => 0,
        }
    }

    /// Sets mouse position
    pub fn set_mouse_position(&mut self, x: u16, y: u16) {
        self.mouse_position = Some((x, y));
    }

    /// Clears mouse position
    pub fn clear_mouse_position(&mut self) {
        self.mouse_position = None;
    }

    /// Cycles to the next theme
    pub fn next_theme(&mut self) {
        use crate::theme::ThemeType;
        self.theme = match self.theme {
            ThemeType::Dark => ThemeType::Light,
            ThemeType::Light => ThemeType::HighContrast,
            ThemeType::HighContrast => ThemeType::SolarizedDark,
            ThemeType::SolarizedDark => ThemeType::SolarizedLight,
            ThemeType::SolarizedLight => ThemeType::Dracula,
            ThemeType::Dracula => ThemeType::Nord,
            ThemeType::Nord => ThemeType::Monokai,
            ThemeType::Monokai => ThemeType::Dark,
        };
    }
}

/// Helper function to format Duration as MM:SS or HH:MM:SS
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_default() {
        assert_eq!(View::default(), View::Library);
    }

    #[test]
    fn test_playback_state_default() {
        let state = PlaybackState::default();
        assert_eq!(state.position, Duration::ZERO);
        assert_eq!(state.duration, Duration::ZERO);
        assert!(!state.is_playing);
        assert_eq!(state.volume, 1.0);
        assert_eq!(state.speed, 1.0);
    }

    #[test]
    fn test_playback_progress() {
        let mut state = PlaybackState::default();
        state.position = Duration::from_secs(30);
        state.duration = Duration::from_secs(100);
        assert_eq!(state.progress(), 0.3);
    }

    #[test]
    fn test_playback_state_remaining() {
        let mut state = PlaybackState::default();
        state.position = Duration::from_secs(30);
        state.duration = Duration::from_secs(100);
        assert_eq!(state.remaining(), Duration::from_secs(70));
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.view, View::Library);
        assert!(!state.should_quit);
    }

    #[test]
    fn test_app_state_view_switching_preserves_selection() {
        let mut state = AppState::new();
        state.library_items_count = 10;

        // Navigate in Library view
        state.select_next();
        state.select_next();
        assert_eq!(state.selected_item, 2);

        // Switch to Player view
        state.set_view(View::Player);
        assert_eq!(state.selected_item, 0); // New view starts at 0

        // Switch back to Library view
        state.set_view(View::Library);
        assert_eq!(state.selected_item, 2); // Should restore previous position
    }

    #[test]
    fn test_app_state_multiple_views_preserve_state() {
        let mut state = AppState::new();
        state.library_items_count = 10;

        // Navigate in Library
        state.select_next();
        state.select_next();
        state.select_next();
        assert_eq!(state.selected_item, 3);

        // Switch to Bookmarks
        state.set_view(View::Bookmarks);
        state.select_next();
        assert_eq!(state.selected_item, 1);

        // Switch to Settings
        state.set_view(View::Settings);
        state.select_next();
        state.select_next();
        assert_eq!(state.selected_item, 2);

        // Return to Library - should restore position 3
        state.set_view(View::Library);
        assert_eq!(state.selected_item, 3);

        // Return to Bookmarks - should restore position 1
        state.set_view(View::Bookmarks);
        assert_eq!(state.selected_item, 1);

        // Return to Settings - should restore position 2
        state.set_view(View::Settings);
        assert_eq!(state.selected_item, 2);
    }

    #[test]
    fn test_app_state_quit() {
        let mut state = AppState::new();
        state.quit();
        assert!(state.should_quit);
    }

    #[test]
    fn test_app_state_status() {
        let mut state = AppState::new();
        state.set_status("Test message");
        assert_eq!(state.status_message, Some("Test message".to_string()));

        state.clear_status();
        assert_eq!(state.status_message, None);
    }

    #[test]
    fn test_app_state_selection() {
        let mut state = AppState::new();
        state.library_items_count = 5;

        state.select_next();
        assert_eq!(state.selected_item, 1);

        state.select_next();
        assert_eq!(state.selected_item, 2);

        state.select_previous();
        assert_eq!(state.selected_item, 1);

        state.reset_selection();
        assert_eq!(state.selected_item, 0);
    }

    #[test]
    fn test_selection_bounds_checking() {
        let mut state = AppState::new();
        state.library_items_count = 3;

        // Can't go below 0
        state.select_previous();
        assert_eq!(state.selected_item, 0);

        // Can't exceed max
        for _ in 0..10 {
            state.select_next();
        }
        assert_eq!(state.selected_item, 2); // Max is 3-1=2
    }

    #[test]
    fn test_format_duration_short() {
        let duration = Duration::from_secs(125); // 2:05
        let formatted = format_duration(duration);
        assert_eq!(formatted, "02:05");
    }

    #[test]
    fn test_format_duration_long() {
        let duration = Duration::from_secs(3665); // 1:01:05
        let formatted = format_duration(duration);
        assert_eq!(formatted, "01:01:05");
    }
}