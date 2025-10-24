// crates/cli/src/tui_mode.rs
//! Integrated TUI mode with real audio playback

use anyhow::{anyhow, Result};
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use media_engine::engine::EngineConfig;
use media_engine::MediaEngine;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use storystream_config::ConfigManager;
use storystream_core::types::book::Book;
use storystream_database::connection::DatabaseConfig;
use storystream_database::queries::books;
use storystream_library::LibraryManager;
use storystream_tui::{AppState, Theme, ThemeType, View};

/// Integrated application
pub struct IntegratedApp {
    tui_state: AppState,
    media_engine: Arc<Mutex<MediaEngine>>,
    theme: Theme,
    current_books: Vec<Book>,
}

impl IntegratedApp {
    /// Create new integrated app
    pub async fn new() -> Result<Self> {
        // Create config manager
        let config_manager = ConfigManager::new()?;
        let config = config_manager.load_or_default();

        // Initialize media engine with correct config
        let engine_config = EngineConfig {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 4096,
        };
        let media_engine = MediaEngine::new(engine_config)
            .map_err(|e| anyhow!("Failed to create media engine: {}", e))?;

        // Create TUI state
        let mut tui_state = AppState::new();
        tui_state.theme = ThemeType::Dark;

        // Load demo books
        let current_books = vec![];

        Ok(Self {
            tui_state,
            media_engine: Arc::new(Mutex::new(media_engine)),
            theme: Theme::new(ThemeType::Dark),
            current_books,
        })
    }

    /// Run the application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main loop
        let result = self.event_loop(&mut terminal).await;

        // Cleanup
        self.cleanup(&mut terminal)?;

        result
    }

    /// Main event loop
    async fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let tick_rate = Duration::from_millis(250);

        loop {
            // Sync state
            self.sync_playback_state()?;

            // Render
            terminal
                .draw(|frame| storystream_tui::ui::render(frame, &self.tui_state, &self.theme))?;

            if self.tui_state.should_quit {
                break;
            }

            // Handle events
            if crossterm::event::poll(tick_rate)? {
                match crossterm::event::read()? {
                    Event::Key(key) => {
                        if key.code == KeyCode::Char('q')
                            || (key.code == KeyCode::Char('c')
                                && key.modifiers.contains(KeyModifiers::CONTROL))
                        {
                            self.tui_state.quit();
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

    /// Sync playback state
    fn sync_playback_state(&mut self) -> Result<()> {
        let engine = self.media_engine.lock().unwrap();
        self.tui_state.playback.position = engine.position();
        self.tui_state.playback.is_playing = engine.is_playing();
        self.tui_state.playback.volume = engine.volume();
        Ok(())
    }

    /// Handle keyboard
    async fn handle_key(&mut self, code: KeyCode) -> Result<()> {
        match code {
            KeyCode::Tab => self.cycle_view(),
            KeyCode::Char('h') => {
                if self.tui_state.view == View::Help {
                    self.tui_state.set_view(View::Library);
                } else {
                    self.tui_state.set_view(View::Help);
                }
            }
            KeyCode::Char('t') => {
                self.tui_state.next_theme();
                self.theme = Theme::new(self.tui_state.theme);
                self.tui_state
                    .set_status(format!("Theme: {}", self.tui_state.theme.name()));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.tui_state.select_previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.tui_state.select_next();
            }
            KeyCode::Char(' ') => {
                if self.tui_state.view == View::Player {
                    let mut engine = self.media_engine.lock().unwrap();
                    if engine.is_playing() {
                        let _ = engine.pause();
                        self.tui_state.set_status("Paused");
                    } else {
                        let _ = engine.play();
                        self.tui_state.set_status("Playing");
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle mouse
    async fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                self.tui_state.select_next();
            }
            MouseEventKind::ScrollUp => {
                self.tui_state.select_previous();
            }
            _ => {}
        }
        Ok(())
    }

    /// Cycle views
    fn cycle_view(&mut self) {
        self.tui_state.view = match self.tui_state.view {
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
        self.tui_state.reset_selection();
    }

    /// Cleanup
    fn cleanup(&self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }
}

/// Run TUI
pub async fn run_tui() -> Result<()> {
    println!("Starting StoryStream TUI...\n");
    std::thread::sleep(Duration::from_secs(1));

    let mut app = IntegratedApp::new().await?;
    app.run().await
}
