// crates/tui/src/state.rs
//! Application state management

use std::time::Duration;

/// Current view in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Library view
    Library,
    /// Now playing view
    Player,
    /// Bookmarks view
    Bookmarks,
    /// Settings view
    Settings,
    /// Help view
    Help,
    /// Search view
    Search,
    /// Playlists view
    Playlists,
    /// Statistics view
    Statistics,
    /// Plugin view
    Plugin,
}

impl Default for View {
    fn default() -> Self {
        Self::Library
    }
}

/// Playback state
#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackState {
    /// Currently playing file
    pub current_file: Option<String>,
    /// Current position in seconds
    pub position: Duration,
    /// Total duration in seconds
    pub duration: Duration,
    /// Is currently playing
    pub is_playing: bool,
    /// Current volume (0.0 - 1.0)
    pub volume: f32,
    /// Playback speed
    pub speed: f32,
    /// Current chapter index
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
    /// Returns progress as a percentage (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if self.duration.as_secs() == 0 {
            0.0
        } else {
            self.position.as_secs_f32() / self.duration.as_secs_f32()
        }
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
        }
    }
}

impl AppState {
    /// Creates a new application state
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the current view
    pub fn set_view(&mut self, view: View) {
        self.view = view;
    }

    /// Requests quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    /// Sets the search query
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    /// Clears the search query
    pub fn clear_search(&mut self) {
        self.search_query.clear();
    }

    /// Sets mouse position
    pub fn set_mouse_position(&mut self, x: u16, y: u16) {
        self.mouse_position = Some((x, y));
    }

    /// Clears mouse position
    pub fn clear_mouse_position(&mut self) {
        self.mouse_position = None;
    }

    /// Sets the theme
    pub fn set_theme(&mut self, theme: crate::theme::ThemeType) {
        self.theme = theme;
    }

    /// Cycles to the next theme
    pub fn next_theme(&mut self) {
        let themes = crate::theme::ThemeType::all();
        let current_index = themes.iter().position(|t| *t == self.theme).unwrap_or(0);
        let next_index = (current_index + 1) % themes.len();
        self.theme = themes[next_index];
    }

    /// Sets status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Clears status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Moves selection up
    pub fn select_previous(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
        }
    }

    /// Moves selection down
    pub fn select_next(&mut self) {
        if self.selected_item + 1 < self.library_items_count {
            self.selected_item += 1;
        }
    }

    /// Resets selection
    pub fn reset_selection(&mut self) {
        self.selected_item = 0;
    }
}

/// Formats a duration as MM:SS or HH:MM:SS
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
        assert!(!state.is_playing);
        assert_eq!(state.volume, 1.0);
    }

    #[test]
    fn test_playback_state_progress() {
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
    fn test_app_state_set_view() {
        let mut state = AppState::new();
        state.set_view(View::Player);
        assert_eq!(state.view, View::Player);
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