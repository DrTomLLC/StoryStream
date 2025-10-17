// crates/tui/src/app.rs
//! Main application logic

use crate::{
    error::TuiResult,
    events::{AppEvent, EventHandler},
    state::{AppState, View},
    theme::Theme,
    ui,
};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use std::time::Duration;

/// The main TUI application
pub struct App {
    state: AppState,
    event_handler: EventHandler,
    theme: Theme,
}

impl App {
    /// Creates a new application
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
            event_handler: EventHandler::new(Duration::from_millis(250)),
            theme: Theme::default(),
        }
    }

    /// Runs the application
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> TuiResult<()> {
        while !self.state.should_quit {
            terminal.draw(|frame| ui::render(frame, &self.state, &self.theme))?;

            match self.event_handler.next()? {
                AppEvent::Key(key) => self.handle_key(key.code, key.modifiers)?,
                AppEvent::Mouse(mouse) => self.handle_mouse(mouse)?,
                AppEvent::Quit => self.state.quit(),
                AppEvent::Tick => self.handle_tick()?,
                AppEvent::Resize(_, _) => {
                    // Terminal will handle resize automatically
                }
            }
        }

        Ok(())
    }

    /// Handles mouse events
    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> TuiResult<()> {
        use crossterm::event::MouseEventKind;

        match mouse.kind {
            MouseEventKind::Down(_) => {
                self.state.set_mouse_position(mouse.column, mouse.row);
            }
            MouseEventKind::Up(_) => {
                self.state.clear_mouse_position();
            }
            MouseEventKind::ScrollDown => {
                self.state.select_next();
            }
            MouseEventKind::ScrollUp => {
                self.state.select_previous();
            }
            _ => {}
        }

        Ok(())
    }

    /// Handles key events
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> TuiResult<()> {
        // Global keys
        match code {
            KeyCode::Char('q') if !modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.quit();
                return Ok(());
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.quit();
                return Ok(());
            }
            KeyCode::Tab => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    self.cycle_view_reverse();
                } else {
                    self.cycle_view();
                }
                return Ok(());
            }
            KeyCode::Char('h') => {
                if self.state.view == View::Help {
                    self.state.set_view(View::Library);
                } else {
                    self.state.set_view(View::Help);
                }
                return Ok(());
            }
            KeyCode::Char('t') => {
                self.state.next_theme();
                self.theme = Theme::new(self.state.theme);
                self.state.set_status(format!("Theme: {}", self.state.theme.name()));
                return Ok(());
            }
            KeyCode::Char('/') => {
                self.state.set_view(View::Search);
                return Ok(());
            }
            KeyCode::Esc => {
                if self.state.view == View::Help {
                    self.state.set_view(View::Library);
                } else if self.state.view == View::Search {
                    self.state.clear_search();
                    self.state.set_view(View::Library);
                }
                return Ok(());
            }
            _ => {}
        }

        // View-specific keys
        match self.state.view {
            View::Library => self.handle_library_keys(code, modifiers)?,
            View::Player => self.handle_player_keys(code, modifiers)?,
            View::Bookmarks => self.handle_bookmarks_keys(code, modifiers)?,
            View::Search => self.handle_search_keys(code, modifiers)?,
            View::Playlists => self.handle_playlists_keys(code, modifiers)?,
            View::Statistics => self.handle_statistics_keys(code, modifiers)?,
            View::Settings => self.handle_settings_keys(code, modifiers)?,
            View::Help => {
                // Any key handled globally
            }
            View::Plugin => {
                // Plugin handling would go here
            }
        }

        Ok(())
    }

    /// Handles library view keys
    fn handle_library_keys(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> TuiResult<()> {
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.select_next();
            }
            KeyCode::Enter => {
                self.state.set_view(View::Player);
                self.state.playback.is_playing = true;
                self.state.playback.current_file = Some("Selected audiobook".to_string());
                self.state.playback.duration = Duration::from_secs(18000); // 5 hours demo
                self.state.set_status("Started playback");
            }
            KeyCode::Char('s') => {
                self.state.set_status("Syncing library...");
            }
            KeyCode::Char('f') => {
                self.state.set_status("Toggled favorite");
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles player view keys
    fn handle_player_keys(&mut self, code: KeyCode, modifiers: KeyModifiers) -> TuiResult<()> {
        match code {
            KeyCode::Char(' ') => {
                self.state.playback.is_playing = !self.state.playback.is_playing;
                let status = if self.state.playback.is_playing {
                    "Playing"
                } else {
                    "Paused"
                };
                self.state.set_status(status);
            }
            KeyCode::Left => {
                let seek_amount = if modifiers.contains(KeyModifiers::SHIFT) {
                    30
                } else {
                    10
                };
                if self.state.playback.position >= Duration::from_secs(seek_amount) {
                    self.state.playback.position -= Duration::from_secs(seek_amount);
                } else {
                    self.state.playback.position = Duration::ZERO;
                }
                self.state.set_status(format!("Seeked backward {}s", seek_amount));
            }
            KeyCode::Right => {
                let seek_amount = if modifiers.contains(KeyModifiers::SHIFT) {
                    30
                } else {
                    10
                };
                self.state.playback.position += Duration::from_secs(seek_amount);
                if self.state.playback.position > self.state.playback.duration {
                    self.state.playback.position = self.state.playback.duration;
                }
                self.state.set_status(format!("Seeked forward {}s", seek_amount));
            }
            KeyCode::Char('[') => {
                if self.state.playback.speed > 0.5 {
                    self.state.playback.speed = (self.state.playback.speed - 0.1).max(0.5);
                    self.state.set_status(format!("Speed: {:.1}x", self.state.playback.speed));
                }
            }
            KeyCode::Char(']') => {
                if self.state.playback.speed < 3.0 {
                    self.state.playback.speed = (self.state.playback.speed + 0.1).min(3.0);
                    self.state.set_status(format!("Speed: {:.1}x", self.state.playback.speed));
                }
            }
            KeyCode::Char('\\') => {
                self.state.playback.speed = 1.0;
                self.state.set_status("Speed reset to 1.0x");
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if self.state.playback.volume < 1.0 {
                    self.state.playback.volume = (self.state.playback.volume + 0.1).min(1.0);
                    self.state.set_status(format!("Volume: {}%", (self.state.playback.volume * 100.0) as u8));
                }
            }
            KeyCode::Char('-') => {
                if self.state.playback.volume > 0.0 {
                    self.state.playback.volume = (self.state.playback.volume - 0.1).max(0.0);
                    self.state.set_status(format!("Volume: {}%", (self.state.playback.volume * 100.0) as u8));
                }
            }
            KeyCode::Char('0') => {
                if self.state.playback.volume > 0.0 {
                    self.state.playback.volume = 0.0;
                    self.state.set_status("Muted");
                } else {
                    self.state.playback.volume = 1.0;
                    self.state.set_status("Unmuted");
                }
            }
            KeyCode::Char('n') => {
                self.state.set_status("Next chapter");
            }
            KeyCode::Char('p') | KeyCode::Char('b') => {
                self.state.set_status("Previous chapter");
            }
            KeyCode::Home => {
                self.state.playback.position = Duration::ZERO;
                self.state.set_status("Jumped to beginning");
            }
            KeyCode::End => {
                self.state.playback.position = self.state.playback.duration;
                self.state.set_status("Jumped to end");
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles bookmarks view keys
    fn handle_bookmarks_keys(&mut self, code: KeyCode, modifiers: KeyModifiers) -> TuiResult<()> {
        match code {
            KeyCode::Char('b') => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    self.state.set_status("Add bookmark with note");
                } else {
                    self.state.set_status("Bookmark added");
                }
            }
            KeyCode::Char('d') => {
                self.state.set_status("Bookmark deleted");
            }
            KeyCode::Char('e') => {
                self.state.set_status("Edit bookmark");
            }
            KeyCode::Up => {
                self.state.select_previous();
            }
            KeyCode::Down => {
                self.state.select_next();
            }
            KeyCode::Enter => {
                self.state.set_status("Jumped to bookmark");
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles search view keys
    fn handle_search_keys(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> TuiResult<()> {
        match code {
            KeyCode::Char(c) => {
                self.state.search_query.push(c);
                self.state.reset_selection();
            }
            KeyCode::Backspace => {
                self.state.search_query.pop();
                self.state.reset_selection();
            }
            KeyCode::Up => {
                self.state.select_previous();
            }
            KeyCode::Down => {
                self.state.select_next();
            }
            KeyCode::Enter => {
                self.state.set_view(View::Player);
                self.state.set_status("Playing selected book");
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles playlists view keys
    fn handle_playlists_keys(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> TuiResult<()> {
        match code {
            KeyCode::Up => {
                self.state.select_previous();
            }
            KeyCode::Down => {
                self.state.select_next();
            }
            KeyCode::Enter => {
                self.state.set_status("Playing playlist");
            }
            KeyCode::Char('n') => {
                self.state.set_status("Create new playlist");
            }
            KeyCode::Char('a') => {
                self.state.set_status("Added to playlist");
            }
            KeyCode::Char('d') => {
                self.state.set_status("Deleted playlist");
            }
            KeyCode::Char('e') => {
                self.state.set_status("Edit playlist");
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles statistics view keys
    fn handle_statistics_keys(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> TuiResult<()> {
        match code {
            KeyCode::Char('r') => {
                self.state.set_status("Refreshed statistics");
            }
            KeyCode::Char('e') => {
                self.state.set_status("Exported statistics to CSV");
            }
            KeyCode::Up => {
                self.state.select_previous();
            }
            KeyCode::Down => {
                self.state.select_next();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles settings view keys
    fn handle_settings_keys(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> TuiResult<()> {
        match code {
            KeyCode::Up => {
                self.state.select_previous();
            }
            KeyCode::Down => {
                self.state.select_next();
            }
            KeyCode::Enter => {
                self.state.set_status("Edit setting");
            }
            KeyCode::Char('r') => {
                self.state.set_status("Reset all settings to defaults");
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles tick events
    fn handle_tick(&mut self) -> TuiResult<()> {
        // Update playback position if playing
        if self.state.playback.is_playing {
            self.state.playback.position += Duration::from_millis(250);
            if self.state.playback.position > self.state.playback.duration {
                self.state.playback.position = self.state.playback.duration;
                self.state.playback.is_playing = false;
            }
        }

        Ok(())
    }

    /// Cycles to the next view
    fn cycle_view(&mut self) {
        self.state.view = match self.state.view {
            View::Library => View::Player,
            View::Player => View::Bookmarks,
            View::Bookmarks => View::Search,
            View::Search => View::Playlists,
            View::Playlists => View::Statistics,
            View::Statistics => View::Settings,
            View::Settings => View::Help,
            View::Help => View::Library,
            View::Plugin => View::Library,
        };
    }

    /// Cycles to the previous view
    fn cycle_view_reverse(&mut self) {
        self.state.view = match self.state.view {
            View::Library => View::Help,
            View::Player => View::Library,
            View::Bookmarks => View::Player,
            View::Search => View::Bookmarks,
            View::Playlists => View::Search,
            View::Statistics => View::Playlists,
            View::Settings => View::Statistics,
            View::Help => View::Settings,
            View::Plugin => View::Help,
        };
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert_eq!(app.state.view, View::Library);
        assert!(!app.state.should_quit);
    }

    #[test]
    fn test_app_default() {
        let app = App::default();
        assert_eq!(app.state.view, View::Library);
    }

    #[test]
    fn test_cycle_view() {
        let mut app = App::new();

        assert_eq!(app.state.view, View::Library);
        app.cycle_view();
        assert_eq!(app.state.view, View::Player);
        app.cycle_view();
        assert_eq!(app.state.view, View::Bookmarks);
        app.cycle_view();
        assert_eq!(app.state.view, View::Search);
        app.cycle_view();
        assert_eq!(app.state.view, View::Playlists);
        app.cycle_view();
        assert_eq!(app.state.view, View::Statistics);
        app.cycle_view();
        assert_eq!(app.state.view, View::Settings);
        app.cycle_view();
        assert_eq!(app.state.view, View::Help);
        app.cycle_view();
        assert_eq!(app.state.view, View::Library);
    }
}