use anyhow::{Context, Result};
use console::{style, Key, Term};
use storystream_core::{Book, Duration as CoreDuration};
use storystream_database::{
    connection::{connect, DatabaseConfig},
    queries::playback::{create_playback_state, get_playback_state, update_playback_state},
    DbPool,
};
use media_engine::{MediaEngine, MediaEvent, PlaybackStatus};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::Mutex;
use tokio::time::interval;

pub async fn start_playback(_db_path: &str, book: Book) -> Result<()> {
    let config = DatabaseConfig::new(_db_path);
    let pool = connect(config)
        .await
        .context("Failed to connect to database")?;

    // Load or create playback state
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

    // Initialize media engine
    let mut engine = MediaEngine::new().context("Failed to initialize media engine")?;

    // Load audio file (note: book_id is Option<i64>, but BookId is UUID)
    engine
        .load(&book.file_path, None)
        .context("Failed to load audio file")?;

    // Restore playback position
    let restore_position = std_duration_from_core(db_state.position);
    if restore_position > StdDuration::ZERO {
        engine
            .seek(restore_position)
            .context("Failed to seek to saved position")?;
    }

    // Restore speed and volume
    engine
        .set_speed(db_state.speed.value())
        .context("Failed to set playback speed")?;

    // Convert volume from 0-100 to 0.0-1.0
    let volume_f32 = db_state.volume as f32 / 100.0;
    engine
        .set_volume(volume_f32)
        .context("Failed to set volume")?;

    // Start playback
    engine.play().context("Failed to start playback")?;

    // Run interactive player
    run_player_ui(&pool, engine, db_state, &book).await?;

    Ok(())
}

async fn run_player_ui(
    pool: &DbPool,
    mut engine: MediaEngine,
    initial_state: storystream_core::PlaybackState,
    book: &Book,
) -> Result<()> {
    let term = Term::stdout();
    term.hide_cursor().context("Failed to hide cursor")?;

    let pool = Arc::new(pool.clone());
    let book_id = book.id;
    let db_state = Arc::new(Mutex::new(initial_state));

    // Spawn background task to save position periodically
    let save_pool = pool.clone();
    let save_state = db_state.clone();
    let save_handle = tokio::spawn(async move {
        let mut ticker = interval(StdDuration::from_secs(5));
        loop {
            ticker.tick().await;
            let current_state = save_state.lock().await;
            if let Err(e) =
                update_playback_state(&save_pool, book_id, current_state.position).await
            {
                eprintln!("Warning: Failed to save position: {}", e);
            }
        }
    });

    // Get event receiver
    let events = engine.events();

    // Spawn event handler
    let event_state = db_state.clone();
    let event_handle = tokio::task::spawn_blocking(move || {
        while let Ok(event) = events.recv() {
            match event {
                MediaEvent::PositionChanged(pos) => {
                    if let Ok(mut state) = event_state.try_lock() {
                        state.position = core_duration_from_std(pos);
                    }
                }
                MediaEvent::StateChanged(_) => {}
                MediaEvent::ChapterChanged(_) => {}
                MediaEvent::PlaybackEnded => {}
                MediaEvent::Error(err) => {
                    eprintln!("Playback error: {}", err);
                }
            }
        }
    });

    let result = player_loop(&term, &mut engine, &db_state, book).await;

    // Cleanup
    save_handle.abort();
    event_handle.abort();
    term.show_cursor().ok();

    // Save final state
    let final_state = db_state.lock().await;
    create_playback_state(&pool, &final_state)
        .await
        .context("Failed to save final playback state")?;

    result
}

async fn player_loop(
    term: &Term,
    engine: &mut MediaEngine,
    db_state: &Arc<Mutex<storystream_core::PlaybackState>>,
    book: &Book,
) -> Result<()> {
    loop {
        // Clear and redraw UI
        term.clear_screen().context("Failed to clear screen")?;

        let current_db_state = db_state.lock().await;
        let engine_state = engine.get_state();
        draw_player_ui(term, &engine_state, &current_db_state, book)?;
        drop(current_db_state);

        // Handle input with timeout
        if let Ok(key) = term.read_key() {
            let mut current_db_state = db_state.lock().await;

            match key {
                Key::Char(' ') => {
                    engine.toggle_playback().context("Failed to toggle playback")?;
                    current_db_state.is_playing = engine.get_state().is_playing();
                }
                Key::Char('q') | Key::Escape => {
                    engine.stop();
                    break;
                }
                Key::ArrowLeft => {
                    engine
                        .skip_backward(StdDuration::from_secs(10))
                        .context("Failed to skip backward")?;
                    current_db_state.position = core_duration_from_std(engine.get_state().position);
                }
                Key::ArrowRight => {
                    engine
                        .skip_forward(StdDuration::from_secs(10))
                        .context("Failed to skip forward")?;
                    current_db_state.position = core_duration_from_std(engine.get_state().position);
                }
                Key::Char('+') | Key::Char('=') => {
                    let current_volume = (current_db_state.volume + 5).min(100);
                    let volume_f32 = current_volume as f32 / 100.0;
                    engine.set_volume(volume_f32).context("Failed to set volume")?;
                    current_db_state.volume = current_volume;
                }
                Key::Char('-') | Key::Char('_') => {
                    let current_volume = current_db_state.volume.saturating_sub(5);
                    let volume_f32 = current_volume as f32 / 100.0;
                    engine.set_volume(volume_f32).context("Failed to set volume")?;
                    current_db_state.volume = current_volume;
                }
                Key::Char('[') => {
                    let current_speed = current_db_state.speed.value();
                    let new_speed = (current_speed - 0.1).max(0.25);
                    engine.set_speed(new_speed).context("Failed to set speed")?;
                    if let Ok(speed) = storystream_core::PlaybackSpeed::new(new_speed) {
                        current_db_state.speed = speed;
                    }
                }
                Key::Char(']') => {
                    let current_speed = current_db_state.speed.value();
                    let new_speed = (current_speed + 0.1).min(3.0);
                    engine.set_speed(new_speed).context("Failed to set speed")?;
                    if let Ok(speed) = storystream_core::PlaybackSpeed::new(new_speed) {
                        current_db_state.speed = speed;
                    }
                }
                Key::Char('n') => {
                    if engine.next_chapter().is_ok() {
                        current_db_state.position =
                            core_duration_from_std(engine.get_state().position);
                    }
                }
                Key::Char('p') => {
                    if engine.previous_chapter().is_ok() {
                        current_db_state.position =
                            core_duration_from_std(engine.get_state().position);
                    }
                }
                _ => {}
            }
        }

        // Small delay to prevent excessive CPU usage
        tokio::time::sleep(StdDuration::from_millis(100)).await;
    }

    Ok(())
}

fn draw_player_ui(
    term: &Term,
    engine_state: &media_engine::PlaybackState,
    db_state: &storystream_core::PlaybackState,
    book: &Book,
) -> Result<()> {
    // Title and author
    term.write_line(&format!("\n  {}", style(&book.title).bold().cyan()))
        .context("Failed to write title")?;

    if let Some(author) = &book.author {
        term.write_line(&format!("  by {}", style(author).dim()))
            .context("Failed to write author")?;
    }

    term.write_line("").context("Failed to write blank line")?;

    // Position and duration
    let position = core_duration_from_std(engine_state.position);
    let duration = book.duration;
    let progress = if duration.as_millis() > 0 {
        (position.as_millis() as f64 / duration.as_millis() as f64 * 100.0) as usize
    } else {
        0
    };

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
    let status = match engine_state.status {
        PlaybackStatus::Playing => style("Playing").green(),
        PlaybackStatus::Paused => style("Paused").yellow(),
        PlaybackStatus::Stopped => style("Stopped").red(),
        PlaybackStatus::Buffering => style("Buffering").cyan(),
    };
    term.write_line(&format!("  Status: {}", status))
        .context("Failed to write status")?;

    term.write_line(&format!("  Speed: {:.2}x", db_state.speed.value()))
        .context("Failed to write speed")?;

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
    term.write_line("    +/-     - Volume up/down")
        .context("Failed to write control")?;
    term.write_line("    [/]     - Speed down/up")
        .context("Failed to write control")?;
    term.write_line("    N/P     - Next/Previous chapter")
        .context("Failed to write control")?;
    term.write_line("    Q/Esc   - Quit")
        .context("Failed to write control")?;

    Ok(())
}

// Helper functions to convert between Duration types
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

    async fn setup_test_db() -> (DbPool, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let config = DatabaseConfig::new(db_path);
        let pool = connect(config).await.unwrap();
        run_migrations(&pool).await.unwrap();

        (pool, temp_file)
    }

    #[tokio::test]
    async fn test_playback_state_persistence() {
        let (pool, _temp) = setup_test_db().await;

        let book = Book::new(
            "Test Book".to_string(),
            PathBuf::from("/test/audio.mp3"),
            1_000_000,
            CoreDuration::from_seconds(3600),
        );

        // IMPORTANT: Create the book in the database first
        books::create_book(&pool, &book)
            .await
            .expect("Failed to create book");

        let state = storystream_core::PlaybackState::new(book.id);
        create_playback_state(&pool, &state)
            .await
            .expect("Failed to create playback state");

        let new_pos = CoreDuration::from_seconds(150);
        update_playback_state(&pool, book.id, new_pos)
            .await
            .expect("Failed to update playback state");

        let retrieved = get_playback_state(&pool, book.id)
            .await
            .expect("Failed to get playback state");
        assert_eq!(retrieved.position, new_pos);
    }

    #[test]
    fn test_duration_conversions() {
        let core_dur = CoreDuration::from_seconds(120);
        let std_dur = std_duration_from_core(core_dur);
        assert_eq!(std_dur.as_secs(), 120);

        let converted_back = core_duration_from_std(std_dur);
        assert_eq!(converted_back.as_seconds(), 120);
    }

    #[tokio::test]
    async fn test_media_engine_initialization() {
        let result = MediaEngine::new();
        assert!(result.is_ok());
    }
}