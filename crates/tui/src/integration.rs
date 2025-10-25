// crates/tui/src/integration.rs
//! Integrated TUI with real functionality
//!
//! This module wires the TUI to the actual StoryStream services:
//! - MediaEngine for audio playback
//! - LibraryManager for book management
//! - Database for persistence
//! - Config for settings

use crate::{error::TuiResult, state::AppState, theme::{Theme, ThemeType}, ui, TuiError};
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind,
};
use crossterm::{execute, terminal::*};
use media_engine::{engine::EngineConfig, MediaEngine, Speed};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};
use storystream_config::{app_config::ColorScheme, ConfigManager};
use storystream_core::types::book::Book;
use storystream_database::{
    connection::{connect, DatabaseConfig},
    queries::books,
    DbPool,
};
use storystream_library::LibraryManager;

/// Convert ColorScheme to ThemeType
fn color_scheme_to_theme(scheme: ColorScheme) -> ThemeType {
    match scheme {
        ColorScheme::Light => ThemeType::Light,
        ColorScheme::Dark => ThemeType::Dark,
        ColorScheme::Auto => ThemeType::Dark, // Default to dark for auto
    }
}

/// Integrated TUI application with real services
pub struct IntegratedTuiApp {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: AppState,
    theme: Theme,
    media_engine: Arc<Mutex<MediaEngine>>,
    library_manager: Arc<LibraryManager>,
    db_pool: DbPool,
    current_books: Vec<Book>,
    tick_rate: Duration,
}

impl IntegratedTuiApp {
    /// Create a new integrated TUI application
    ///
    /// # Errors
    ///
    /// Returns `TuiError` if initialization fails for any component
    pub async fn new() -> TuiResult<Self> {
        // Load configuration
        let config_manager = ConfigManager::new()
            .map_err(|e| TuiError::Initialization(format!("Config error: {}", e)))?;
        let config = config_manager.load_or_default();

        // Initialize database
        let db_config = DatabaseConfig::new(&config.library.database_path);
        let db_pool = connect(db_config)
            .await
            .map_err(|e| TuiError::Initialization(format!("Database error: {}", e)))?;

        // Initialize media engine
        let engine_config = EngineConfig {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 4096,
        };
        let media_engine = MediaEngine::new(engine_config)
            .map_err(|e| TuiError::Initialization(format!("Media engine error: {}", e)))?;
        let media_engine = Arc::new(Mutex::new(media_engine));

        // Initialize library manager
        let library_config = storystream_library::LibraryConfig {
            database_path: config.library.database_path.clone(),
            watch_directories: config.library.paths.clone(),
            auto_import: config.library.auto_import,
        };
        let library_manager = LibraryManager::new(library_config)
            .await
            .map_err(|e| TuiError::Initialization(format!("Library error: {}", e)))?;
        let library_manager = Arc::new(library_manager);

        // Load books from database
        let current_books = books::list_books(&db_pool)
            .await
            .map_err(|e| TuiError::Initialization(format!("Failed to load books: {}", e)))?;

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Initialize TUI state
        let mut state = AppState::new();
        state.library_items_count = current_books.len();
        state.theme = color_scheme_to_theme(config.app.color_scheme);

        Ok(Self {
            terminal,
            state,
            theme: Theme::new(color_scheme_to_theme(config.app.color_scheme)),
            media_engine,
            library_manager,
            db_pool,
            current_books,
            tick_rate: Duration::from_millis(250),
        })
    }

    /// Run the integrated TUI application
    ///
    /// # Errors
    ///
    /// Returns `TuiError` if the event loop encounters an error
    pub async fn run(&mut self) -> TuiResult<()> {
        let result = self.event_loop().await;
        self.cleanup()?;
        result
    }

    /// Main event loop
    async fn event_loop(&mut self) -> TuiResult<()> {
        loop {
            // Sync playback state from media engine
            self.sync_playback_state()?;

            // Update library items count
            self.state.library_items_count = self.current_books.len();

            // Render UI
            self.terminal
                .draw(|frame| ui::render(frame, &self.state, &self.theme))?;

            // Check if we should quit
            if self.state.should_quit {
                break;
            }

            // Handle events with timeout
            if crossterm::event::poll(self.tick_rate)? {
                match crossterm::event::read()? {
                    Event::Key(key) => {
                        // Handle quit commands
                        if key.code == KeyCode::Char('q')
                            || (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL))
                        {
                            self.state.quit();
                            continue;
                        }
                        self.handle_key(key.code).await?;
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse(mouse).await?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Sync playback state from media engine
    fn sync_playback_state(&mut self) -> TuiResult<()> {
        let engine = self
            .media_engine
            .lock()
            .map_err(|e| TuiError::PlaybackError(format!("Lock error: {}", e)))?;

        self.state.playback.position = engine.position();
        self.state.playback.is_playing = engine.is_playing();
        self.state.playback.volume = engine.volume();

        // Get speed value from Arc<Mutex<Speed>>
        if let Ok(speed_guard) = engine.speed.lock() {
            self.state.playback.speed = speed_guard.value();
        }

        // Get duration if we have a current file
        if self.state.playback.current_file.is_some() {
            if let Some(duration) = engine.duration {
                self.state.playback.duration = duration;
            }
        }

        Ok(())
    }

    /// Handle keyboard input
    async fn handle_key(&mut self, code: KeyCode) -> TuiResult<()> {
        match code {
            KeyCode::Tab => self.cycle_view(),
            KeyCode::Char('h') => self.toggle_help(),
            KeyCode::Char('t') => self.toggle_theme(),
            KeyCode::Up | KeyCode::Char('k') => self.state.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self.state.select_next(),
            KeyCode::Char(' ') => self.toggle_playback().await?,
            KeyCode::Enter => self.handle_select().await?,
            KeyCode::Left => self.seek_backward().await?,
            KeyCode::Right => self.seek_forward().await?,
            KeyCode::Char('+') | KeyCode::Char('=') => self.volume_up().await?,
            KeyCode::Char('-') | KeyCode::Char('_') => self.volume_down().await?,
            KeyCode::Char('[') => self.speed_down().await?,
            KeyCode::Char(']') => self.speed_up().await?,
            _ => {}
        }
        Ok(())
    }

    /// Handle mouse input
    async fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> TuiResult<()> {
        match mouse.kind {
            MouseEventKind::ScrollDown => self.state.select_next(),
            MouseEventKind::ScrollUp => self.state.select_previous(),
            MouseEventKind::Down(_) => {
                // Handle click events based on view
                // TODO: Implement click-to-select based on row
            }
            _ => {}
        }
        Ok(())
    }

    /// Cycle to next view
    fn cycle_view(&mut self) {
        use crate::state::View;

        let next_view = match self.state.view {
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

        self.state.set_view(next_view);
        self.state
            .set_status(format!("Switched to {:?} view", next_view));
    }

    /// Toggle help view
    fn toggle_help(&mut self) {
        use crate::state::View;

        if self.state.view == View::Help {
            self.state.set_view(View::Library);
        } else {
            self.state.set_view(View::Help);
        }
    }

    /// Toggle theme
    fn toggle_theme(&mut self) {
        self.state.next_theme();
        self.theme = Theme::new(self.state.theme);
        self.state
            .set_status(format!("Theme: {:?}", self.state.theme));
    }

    /// Toggle playback
    async fn toggle_playback(&mut self) -> TuiResult<()> {
        let mut engine = self
            .media_engine
            .lock()
            .map_err(|e| TuiError::PlaybackError(format!("Lock error: {}", e)))?;

        if engine.is_playing() {
            engine
                .pause()
                .map_err(|e| TuiError::PlaybackError(format!("Pause error: {}", e)))?;
            self.state.set_status("Paused");
        } else {
            engine
                .play()
                .map_err(|e| TuiError::PlaybackError(format!("Play error: {}", e)))?;
            self.state.set_status("Playing");
        }

        Ok(())
    }

    /// Handle selection in current view
    async fn handle_select(&mut self) -> TuiResult<()> {
        use crate::state::View;

        match self.state.view {
            View::Library => {
                // Clone the book to avoid borrow checker issues
                if let Some(book) = self.current_books.get(self.state.selected_item).cloned() {
                    self.load_book(&book).await?;
                }
            }
            View::Player => {
                self.toggle_playback().await?;
            }
            _ => {
                self.state.set_status("Selection not implemented for this view");
            }
        }

        Ok(())
    }

    /// Load and play a book
    ///
    /// # Errors
    ///
    /// Returns `TuiError::PlaybackError` if loading or playing fails
    async fn load_book(&mut self, book: &Book) -> TuiResult<()> {
        let mut engine = self
            .media_engine
            .lock()
            .map_err(|e| TuiError::PlaybackError(format!("Lock error: {}", e)))?;

        // Load the audio file (book.file_path is PathBuf, load expects &PathBuf)
        engine
            .load(&book.file_path)
            .map_err(|e| TuiError::PlaybackError(format!("Load error: {}", e)))?;

        self.state.playback.current_file = Some(book.title.clone());

        // Get duration from the engine after loading
        if let Some(duration) = engine.duration {
            self.state.playback.duration = duration;
        }
        self.state.playback.position = Duration::ZERO;

        engine
            .play()
            .map_err(|e| TuiError::PlaybackError(format!("Play error: {}", e)))?;

        self.state.set_view(crate::state::View::Player);
        self.state.set_status(format!("Playing: {}", book.title));

        Ok(())
    }

    /// Seek backward
    async fn seek_backward(&mut self) -> TuiResult<()> {
        let mut engine = self
            .media_engine
            .lock()
            .map_err(|e| TuiError::PlaybackError(format!("Lock error: {}", e)))?;

        let current = engine.position();
        let new_position = current.saturating_sub(Duration::from_secs(10));

        engine
            .seek(new_position)
            .map_err(|e| TuiError::PlaybackError(format!("Seek error: {}", e)))?;

        self.state.set_status("Seek -10s");
        Ok(())
    }

    /// Seek forward
    async fn seek_forward(&mut self) -> TuiResult<()> {
        let mut engine = self
            .media_engine
            .lock()
            .map_err(|e| TuiError::PlaybackError(format!("Lock error: {}", e)))?;

        let current = engine.position();
        let duration = engine.duration.unwrap_or(Duration::from_secs(0));
        let new_position = (current + Duration::from_secs(10)).min(duration);

        engine
            .seek(new_position)
            .map_err(|e| TuiError::PlaybackError(format!("Seek error: {}", e)))?;

        self.state.set_status("Seek +10s");
        Ok(())
    }

    /// Increase volume
    async fn volume_up(&mut self) -> TuiResult<()> {
        let mut engine = self
            .media_engine
            .lock()
            .map_err(|e| TuiError::PlaybackError(format!("Lock error: {}", e)))?;

        let new_volume = (engine.volume() + 0.1).min(1.0);
        engine
            .set_volume(new_volume)
            .map_err(|e| TuiError::PlaybackError(format!("Volume error: {}", e)))?;

        self.state
            .set_status(format!("Volume: {}%", (new_volume * 100.0) as u8));
        Ok(())
    }

    /// Decrease volume
    async fn volume_down(&mut self) -> TuiResult<()> {
        let mut engine = self
            .media_engine
            .lock()
            .map_err(|e| TuiError::PlaybackError(format!("Lock error: {}", e)))?;

        let new_volume = (engine.volume() - 0.1).max(0.0);
        engine
            .set_volume(new_volume)
            .map_err(|e| TuiError::PlaybackError(format!("Volume error: {}", e)))?;

        self.state
            .set_status(format!("Volume: {}%", (new_volume * 100.0) as u8));
        Ok(())
    }

    /// Decrease playback speed
    async fn speed_down(&mut self) -> TuiResult<()> {
        let mut engine = self
            .media_engine
            .lock()
            .map_err(|e| TuiError::PlaybackError(format!("Lock error: {}", e)))?;

        // Get current speed
        let current_speed = if let Ok(speed_guard) = engine.speed.lock() {
            speed_guard.value()
        } else {
            1.0
        };

        let new_speed_value = (current_speed - 0.1).max(0.5);
        let new_speed = Speed::new(new_speed_value)
            .map_err(|e| TuiError::PlaybackError(format!("Invalid speed: {}", e)))?;

        engine
            .set_speed(new_speed)
            .map_err(|e| TuiError::PlaybackError(format!("Speed error: {}", e)))?;

        self.state
            .set_status(format!("Speed: {:.1}x", new_speed_value));
        Ok(())
    }

    /// Increase playback speed
    async fn speed_up(&mut self) -> TuiResult<()> {
        let mut engine = self
            .media_engine
            .lock()
            .map_err(|e| TuiError::PlaybackError(format!("Lock error: {}", e)))?;

        // Get current speed
        let current_speed = if let Ok(speed_guard) = engine.speed.lock() {
            speed_guard.value()
        } else {
            1.0
        };

        let new_speed_value = (current_speed + 0.1).min(2.0);
        let new_speed = Speed::new(new_speed_value)
            .map_err(|e| TuiError::PlaybackError(format!("Invalid speed: {}", e)))?;

        engine
            .set_speed(new_speed)
            .map_err(|e| TuiError::PlaybackError(format!("Speed error: {}", e)))?;

        self.state
            .set_status(format!("Speed: {:.1}x", new_speed_value));
        Ok(())
    }

    /// Cleanup terminal state
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

impl Drop for IntegratedTuiApp {
    fn drop(&mut self) {
        // Cleanup is safe to fail in drop
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme_conversion() {
        assert_eq!(color_scheme_to_theme(ColorScheme::Light), ThemeType::Light);
        assert_eq!(color_scheme_to_theme(ColorScheme::Dark), ThemeType::Dark);
        assert_eq!(color_scheme_to_theme(ColorScheme::Auto), ThemeType::Dark);
    }
}