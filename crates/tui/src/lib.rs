// crates/tui/src/lib.rs
//! Terminal User Interface for StoryStream
//!
//! This module provides a complete, interactive TUI for the StoryStream audiobook player.
//!
//! # Features
//!
//! - Library browsing with keyboard navigation
//! - Now playing screen with progress bar
//! - Playback controls (play/pause, seek, speed, volume)
//! - Chapter navigation
//! - Bookmark management
//! - Search functionality
//! - Playlists
//! - Statistics dashboard
//! - Settings configuration
//! - Help screen with keyboard shortcuts
//! - Mouse support
//! - Configurable color themes
//! - Plugin system
//!
//! # Example
//!
//! ```rust,no_run
//! use storystream_tui::TuiApp;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut app = TuiApp::new()?;
//! app.run()?;
//! # Ok(())
//! # }
//! ```

mod app;
mod error;
mod events;
mod plugins;
mod state;
mod theme;
mod ui;

pub use app::App;
pub use error::{TuiError, TuiResult};
pub use plugins::{Plugin, PluginManager};
pub use state::{AppState, PlaybackState, View};
pub use theme::{Theme, ThemeType};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

/// Main TUI application wrapper
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    app: App,
}

impl TuiApp {
    /// Creates and initializes a new TUI application
    pub fn new() -> TuiResult<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            app: App::new(),
        })
    }

    /// Runs the TUI application
    pub fn run(&mut self) -> TuiResult<()> {
        let result = self.app.run(&mut self.terminal);
        self.cleanup()?;
        result
    }

    /// Cleans up terminal state
    fn cleanup(&mut self) -> TuiResult<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_exports_accessible() {
        let _: AppState = AppState::new();
        let _: View = View::Library;
        let _: ThemeType = ThemeType::Dark;
    }

    #[test]
    fn test_app_creation() {
        let app = App::new();
        let _ = app;
    }
}