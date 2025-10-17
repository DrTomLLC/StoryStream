// crates/tui/src/lib.rs
//! Terminal User Interface for StoryStream

mod app;
mod error;
mod events;
mod plugins;
mod state;
mod theme;
pub mod ui;  // CHANGED: Made public

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