use anyhow::{Context, Result};
use console::{style, Key, Term};
use storystream_core::{Book, Duration as CoreDuration};
use storystream_database::{
    connection::{connect, DatabaseConfig},
    queries::playback::{create_playback_state, get_playback_state, update_playback_state},
    queries::chapters::get_book_chapters,
    DbPool,
};
use media_engine::{ChapterMarker, MediaEngine, PlaybackStatus};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::Mutex;
use tokio::time::interval;

pub async fn start_playback(_db_path: &str, book: Book) -> Result<()> {
    let config = DatabaseConfig::new(_db_path);
    let pool = connect(config)
        .await
        .context("Failed to connect to database")?;

    // Get or create playback state from database
    let db_state = match get_playback_state(&pool, book.id).await {
        Ok(s) => s,
        Err(_) => {
            let new_state = storystream_core::PlaybackState::new(book.id);
            create_playback_state(&pool, &new_state)
                .await
                .context("Failed to create initial playback state")?;
            new_state
        }
    };

    // Create and configure media engine
    let mut engine = MediaEngine::new().context("Failed to create media engine")?;

    engine
        .load(&book.file_path)
        .context("Failed to load audio file")?;

    // Load chapters from database if available
    if let Ok(db_chapters) = get_book_chapters(&pool, book.id).await {
        let chapters: Vec<ChapterMarker> = db_chapters
            .into_iter()
            .enumerate()
            .map(|(idx, ch)| {
                ChapterMarker::new(
                    idx,
                    ch.title,
                    ch.start_time.as_millis() as f64 / 1000.0,
                    ch.end_time.as_millis() as f64 / 1000.0,
                )
            })
            .collect();

        if !chapters.is_empty() {
            engine.load_chapters(chapters)?;
        }
    }

    // Restore saved position
    let restore_position_ms = db_state.position.as_millis();
    if restore_position_ms > 0 {
        let restore_position_secs = restore_position_ms as f64 / 1000.0;
        engine
            .seek(restore_position_secs)
            .context("Failed to seek to saved position")?;
    }

    // Restore saved speed
    if let Ok(speed) = media_engine::PlaybackSpeed::new(db_state.speed.value()) {
        let _ = engine.set_speed(speed);
    }

    // Restore saved volume (convert from 0-100 to 0.0-1.0)
    let _ = engine.set_volume(db_state.volume as f32 / 100.0);

    // Start playback
    engine.play().context("Failed to start playback")?;

    // Run the player UI
    run_player_ui(&pool, engine, db_state, &book).await?;

    Ok(())
}

async fn run_player_ui(
    pool: &DbPool,
    engine: MediaEngine,
    initial_state: storystream_core::PlaybackState,
    book: &Book,
) -> Result<()> {
    let term = Term::stdout();
    if term.hide_cursor().is_err() {
        eprintln!("Warning: Failed to hide cursor");
    }

    let pool = Arc::new(pool.clone());
    let book_id = book.id;
    let db_state = Arc::new(Mutex::new(initial_state));
    let engine = Arc::new(Mutex::new(engine));

    // Background task: Save position to database every 5 seconds
    let save_pool = pool.clone();
    let save_state = db_state.clone();
    let save_engine = engine.clone();
    let save_handle = tokio::spawn(async move {
        let mut ticker = interval(StdDuration::from_secs(5));
        loop {
            ticker.tick().await;

            // Get current position from engine
            let current_position = if let Ok(eng) = save_engine.try_lock() {
                eng.position()
            } else {
                continue;
            };

            // Update database state with current position
            if let Ok(mut state) = save_state.try_lock() {
                state.position = core_duration_from_std(StdDuration::from_secs_f64(current_position));

                if let Err(e) = update_playback_state(&save_pool, book_id, state.position).await {
                    eprintln!("Warning: Failed to save position: {}", e);
                }
            }
        }
    });

    // Main player loop
    let result = player_loop(&term, &engine, &db_state, book).await;

    // Stop background save task
    save_handle.abort();

    // Show cursor again
    let _ = term.show_cursor();

    // Save final state before exiting
    if let Ok(eng) = engine.try_lock() {
        let final_position = eng.position();

        if let Ok(mut state) = db_state.try_lock() {
            state.position = core_duration_from_std(StdDuration::from_secs_f64(final_position));
            state.is_playing = false;

            if let Err(e) = create_playback_state(&pool, &state).await {
                eprintln!("Warning: Failed to save final state: {}", e);
            }
        }
    }

    result
}

async fn player_loop(
    term: &Term,
    engine: &Arc<Mutex<MediaEngine>>,
    db_state: &Arc<Mutex<storystream_core::PlaybackState>>,
    book: &Book,
) -> Result<()> {
    let mut ui_ticker = interval(StdDuration::from_millis(100));

    loop {
        ui_ticker.tick().await;

        // Clear screen
        if term.clear_screen().is_err() {
            break;
        }

        // Draw UI
        match (engine.try_lock(), db_state.try_lock()) {
            (Ok(eng), Ok(state)) => {
                if draw_player_ui(term, &eng, &state, book).is_err() {
                    break;
                }
            }
            _ => {
                tokio::time::sleep(StdDuration::from_millis(50)).await;
                continue;
            }
        }

        // Handle keyboard input (non-blocking)
        if let Ok(key) = term.read_key() {
            if handle_key_press(key, engine, db_state).await? {
                break; // User requested quit
            }
        }
    }

    Ok(())
}

async fn handle_key_press(
    key: Key,
    engine: &Arc<Mutex<MediaEngine>>,
    db_state: &Arc<Mutex<storystream_core::PlaybackState>>,
) -> Result<bool> {
    let should_quit = match key {
        Key::Char(' ') => {
            // Toggle play/pause
            if let (Ok(mut eng), Ok(mut state)) = (engine.try_lock(), db_state.try_lock()) {
                let status = eng.status();
                if status == PlaybackStatus::Playing {
                    let _ = eng.pause();
                    state.is_playing = false;
                } else {
                    let _ = eng.play();
                    state.is_playing = true;
                }
            }
            false
        }
        Key::Char('q') | Key::Escape => {
            // Quit
            if let Ok(mut eng) = engine.try_lock() {
                let _ = eng.stop();
            }
            true
        }
        Key::ArrowLeft => {
            // Seek backward 10 seconds
            if let (Ok(mut eng), Ok(mut state)) = (engine.try_lock(), db_state.try_lock()) {
                let pos = eng.position();
                let new_pos = (pos - 10.0).max(0.0);
                if eng.seek(new_pos).is_ok() {
                    state.position = core_duration_from_std(StdDuration::from_secs_f64(new_pos));
                }
            }
            false
        }
        Key::ArrowRight => {
            // Seek forward 10 seconds
            if let (Ok(mut eng), Ok(mut state)) = (engine.try_lock(), db_state.try_lock()) {
                let pos = eng.position();
                let new_pos = pos + 10.0;
                if eng.seek(new_pos).is_ok() {
                    state.position = core_duration_from_std(StdDuration::from_secs_f64(new_pos));
                }
            }
            false
        }
        Key::Char('n') | Key::Char('N') => {
            // Next chapter
            if let (Ok(mut eng), Ok(mut state)) = (engine.try_lock(), db_state.try_lock()) {
                if eng.next_chapter().is_ok() {
                    let new_pos = eng.position();
                    state.position = core_duration_from_std(StdDuration::from_secs_f64(new_pos));
                }
            }
            false
        }
        Key::Char('p') | Key::Char('P') => {
            // Previous chapter
            if let (Ok(mut eng), Ok(mut state)) = (engine.try_lock(), db_state.try_lock()) {
                if eng.previous_chapter().is_ok() {
                    let new_pos = eng.position();
                    state.position = core_duration_from_std(StdDuration::from_secs_f64(new_pos));
                }
            }
            false
        }
        Key::Char('+') | Key::Char('=') => {
            // Volume up
            if let (Ok(mut eng), Ok(mut state)) = (engine.try_lock(), db_state.try_lock()) {
                let new_volume = (state.volume + 5).min(100);
                if eng.set_volume(new_volume as f32 / 100.0).is_ok() {
                    state.volume = new_volume;
                }
            }
            false
        }
        Key::Char('-') | Key::Char('_') => {
            // Volume down
            if let (Ok(mut eng), Ok(mut state)) = (engine.try_lock(), db_state.try_lock()) {
                let new_volume = state.volume.saturating_sub(5);
                if eng.set_volume(new_volume as f32 / 100.0).is_ok() {
                    state.volume = new_volume;
                }
            }
            false
        }
        Key::Char('[') => {
            // Speed down
            if let (Ok(mut eng), Ok(mut state)) = (engine.try_lock(), db_state.try_lock()) {
                let current_speed = state.speed.value();
                let new_speed = (current_speed - 0.1).max(0.5);
                if let Ok(speed) = media_engine::PlaybackSpeed::new(new_speed) {
                    if eng.set_speed(speed).is_ok() {
                        if let Ok(core_speed) = storystream_core::PlaybackSpeed::new(new_speed) {
                            state.speed = core_speed;
                        }
                    }
                }
            }
            false
        }
        Key::Char(']') => {
            // Speed up
            if let (Ok(mut eng), Ok(mut state)) = (engine.try_lock(), db_state.try_lock()) {
                let current_speed = state.speed.value();
                let new_speed = (current_speed + 0.1).min(3.0);
                if let Ok(speed) = media_engine::PlaybackSpeed::new(new_speed) {
                    if eng.set_speed(speed).is_ok() {
                        if let Ok(core_speed) = storystream_core::PlaybackSpeed::new(new_speed) {
                            state.speed = core_speed;
                        }
                    }
                }
            }
            false
        }
        _ => false,
    };

    Ok(should_quit)
}

fn draw_player_ui(
    term: &Term,
    engine: &MediaEngine,
    db_state: &storystream_core::PlaybackState,
    book: &Book,
) -> Result<()> {
    // Title
    term.write_line(&format!("\n  {}", style(&book.title).bold().cyan()))
        .context("Failed to write title")?;

    // Author
    if let Some(author) = &book.author {
        term.write_line(&format!("  by {}", style(author).dim()))
            .context("Failed to write author")?;
    }

    term.write_line("").context("Failed to write blank line")?;

    // Chapter info if available
    if engine.has_chapters() {
        let chapter_info = if let Some(ch) = engine.current_chapter() {
            format!("  {} - {}",
                    style(engine.chapter_progress()).yellow(),
                    style(&ch.title).dim())
        } else {
            format!("  {}", style(engine.chapter_progress()).yellow())
        };
        term.write_line(&chapter_info).context("Failed to write chapter info")?;
    }

    // Get current position from engine (real-time)
    let position = core_duration_from_std(StdDuration::from_secs_f64(engine.position()));
    let duration = book.duration;

    // Calculate progress percentage
    let progress = if duration.as_millis() > 0 {
        (position.as_millis() as f64 / duration.as_millis() as f64 * 100.0) as usize
    } else {
        0
    };

    // Time display
    term.write_line(&format!("  {} / {}", position.as_hms(), duration.as_hms()))
        .context("Failed to write position")?;

    // Progress bar
    let bar_width = 50;
    let filled = (progress * bar_width / 100).min(bar_width);
    let bar = format!(
        "  [{}{}] {}%",
        "=".repeat(filled),
        " ".repeat(bar_width - filled),
        progress
    );
    term.write_line(&bar)
        .context("Failed to write progress bar")?;
    term.write_line("").context("Failed to write blank line")?;

    // Status
    let status = match engine.status() {
        PlaybackStatus::Playing => style("Playing").green(),
        PlaybackStatus::Paused => style("Paused").yellow(),
        PlaybackStatus::Stopped => style("Stopped").red(),
    };

    term.write_line(&format!("  Status: {}", status))
        .context("Failed to write status")?;

    // Speed
    term.write_line(&format!("  Speed: {:.2}x", db_state.speed.value()))
        .context("Failed to write speed")?;

    // Volume
    term.write_line(&format!("  Volume: {}%", db_state.volume))
        .context("Failed to write volume")?;

    term.write_line("").context("Failed to write blank line")?;

    // Controls
    term.write_line("  Controls:")
        .context("Failed to write controls header")?;
    term.write_line("    Space   - Play/Pause")
        .context("Failed to write control")?;
    term.write_line("    ←/→     - Seek -10s/+10s")
        .context("Failed to write control")?;

    // Show chapter navigation if chapters available
    if engine.has_chapters() {
        term.write_line("    N/P     - Next/Previous Chapter")
            .context("Failed to write control")?;
    }

    term.write_line("    +/-     - Volume up/down")
        .context("Failed to write control")?;
    term.write_line("    [/]     - Speed down/up")
        .context("Failed to write control")?;
    term.write_line("    Q/Esc   - Quit")
        .context("Failed to write control")?;

    Ok(())
}

fn std_duration_from_core(d: CoreDuration) -> StdDuration {
    StdDuration::from_millis(d.as_millis())
}

fn core_duration_from_std(d: StdDuration) -> CoreDuration {
    CoreDuration::from_millis(d.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use storystream_database::migrations::run_migrations;
    use storystream_database::queries::books;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    async fn setup_test_db() -> Result<(DbPool, NamedTempFile)> {
        let temp_file = NamedTempFile::new()?;
        let db_path = temp_file.path().to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;

        let config = DatabaseConfig::new(db_path);
        let pool = connect(config).await?;
        run_migrations(&pool).await?;

        Ok((pool, temp_file))
    }

    #[tokio::test]
    async fn test_playback_state_persistence() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;

        let book = Book::new(
            "Test Book".to_string(),
            PathBuf::from("/test/audio.mp3"),
            1_000_000,
            CoreDuration::from_seconds(3600),
        );

        books::create_book(&pool, &book).await?;

        let state = storystream_core::PlaybackState::new(book.id);
        create_playback_state(&pool, &state).await?;

        let new_pos = CoreDuration::from_seconds(150);
        update_playback_state(&pool, book.id, new_pos).await?;

        let retrieved = get_playback_state(&pool, book.id).await?;
        assert_eq!(retrieved.position, new_pos);

        Ok(())
    }

    #[test]
    fn test_duration_conversions() {
        let core_dur = CoreDuration::from_seconds(120);
        let std_dur = std_duration_from_core(core_dur);
        assert_eq!(std_dur.as_secs(), 120);

        let converted_back = core_duration_from_std(std_dur);
        assert_eq!(converted_back.as_seconds(), 120);
    }

    #[test]
    fn test_media_engine_initialization() {
        let engine = MediaEngine::new().expect("Failed to create engine");
        assert_eq!(engine.status(), PlaybackStatus::Stopped);
    }

    #[test]
    fn test_position_tracking_accuracy() {
        // Test that conversions maintain precision
        let positions = vec![0, 1000, 5000, 10000, 60000, 3600000];

        for pos_ms in positions {
            let core_dur = CoreDuration::from_millis(pos_ms);
            let std_dur = std_duration_from_core(core_dur);
            let back = core_duration_from_std(std_dur);

            assert_eq!(back.as_millis(), pos_ms);
        }
    }

    #[test]
    fn test_chapter_support() {
        let engine = MediaEngine::new().expect("Failed to create engine");
        assert!(!engine.has_chapters());
        assert_eq!(engine.chapter_progress(), "No chapters");
    }
}