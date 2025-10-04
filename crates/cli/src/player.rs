use anyhow::{Context, Result};
use console::{style, Key, Term};
use storystream_core::{Book, Duration as CoreDuration};
use storystream_database::{
    connection::{connect, DatabaseConfig},
    queries::playback::{create_playback_state, get_playback_state, update_playback_state},
    DbPool,
};
use media_engine::{AudioEngine, PlaybackStatus};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::Mutex;
use tokio::time::interval;

pub async fn start_playback(_db_path: &str, book: Book) -> Result<()> {
    let config = DatabaseConfig::new(_db_path);
    let pool = connect(config)
        .await
        .context("Failed to connect to database")?;

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

    let engine = AudioEngine::new();

    engine
        .load(&book.file_path)
        .context("Failed to load audio file")?;

    let restore_position = std_duration_from_core(db_state.position);
    if restore_position > StdDuration::ZERO {
        engine
            .seek(restore_position)
            .context("Failed to seek to saved position")?;
    }

    if let Ok(speed) = media_engine::PlaybackSpeed::new(db_state.speed.value()) {
        let _ = engine.set_speed(speed);
    }

    let _ = engine.set_volume(db_state.volume);

    engine.play().context("Failed to start playback")?;

    run_player_ui(&pool, engine, db_state, &book).await?;

    Ok(())
}

async fn run_player_ui(
    pool: &DbPool,
    engine: AudioEngine,
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
    let engine = Arc::new(engine);

    let save_pool = pool.clone();
    let save_state = db_state.clone();
    let save_handle = tokio::spawn(async move {
        let mut ticker = interval(StdDuration::from_secs(5));
        loop {
            ticker.tick().await;

            match save_state.try_lock() {
                Ok(current_state) => {
                    if let Err(e) = update_playback_state(&save_pool, book_id, current_state.position).await {
                        eprintln!("Warning: Failed to save position: {}", e);
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
    });

    let result = player_loop(&term, &engine, &db_state, book).await;

    save_handle.abort();
    let _ = term.show_cursor();

    match db_state.try_lock() {
        Ok(final_state) => {
            if let Err(e) = create_playback_state(&pool, &final_state).await {
                eprintln!("Warning: Failed to save final state: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to acquire lock for final save: {}", e);
        }
    }

    result
}

async fn player_loop(
    term: &Term,
    engine: &Arc<AudioEngine>,
    db_state: &Arc<Mutex<storystream_core::PlaybackState>>,
    book: &Book,
) -> Result<()> {
    loop {
        if term.clear_screen().is_err() {
            break;
        }

        match db_state.try_lock() {
            Ok(current_db_state) => {
                if draw_player_ui(term, engine, &current_db_state, book).is_err() {
                    break;
                }
            }
            Err(_) => {
                tokio::time::sleep(StdDuration::from_millis(50)).await;
                continue;
            }
        }

        if let Ok(key) = term.read_key() {
            match db_state.try_lock() {
                Ok(mut current_db_state) => {
                    match key {
                        Key::Char(' ') => {
                            if let Ok(status) = engine.status() {
                                if status == PlaybackStatus::Playing {
                                    let _ = engine.pause();
                                    current_db_state.is_playing = false;
                                } else {
                                    let _ = engine.play();
                                    current_db_state.is_playing = true;
                                }
                            }
                        }
                        Key::Char('q') | Key::Escape => {
                            let _ = engine.stop();
                            break;
                        }
                        Key::ArrowLeft => {
                            if let Ok(pos) = engine.position() {
                                let new_pos = pos.saturating_sub(StdDuration::from_secs(10));
                                if engine.seek(new_pos).is_ok() {
                                    current_db_state.position = core_duration_from_std(new_pos);
                                }
                            }
                        }
                        Key::ArrowRight => {
                            if let Ok(pos) = engine.position() {
                                let new_pos = pos + StdDuration::from_secs(10);
                                if engine.seek(new_pos).is_ok() {
                                    current_db_state.position = core_duration_from_std(new_pos);
                                }
                            }
                        }
                        Key::Char('+') | Key::Char('=') => {
                            let current_volume = (current_db_state.volume + 5).min(100);
                            if engine.set_volume(current_volume).is_ok() {
                                current_db_state.volume = current_volume;
                            }
                        }
                        Key::Char('-') | Key::Char('_') => {
                            let current_volume = current_db_state.volume.saturating_sub(5);
                            if engine.set_volume(current_volume).is_ok() {
                                current_db_state.volume = current_volume;
                            }
                        }
                        Key::Char('[') => {
                            let current_speed = current_db_state.speed.value();
                            let new_speed = (current_speed - 0.1).max(0.5);
                            if let Ok(speed) = media_engine::PlaybackSpeed::new(new_speed) {
                                if engine.set_speed(speed).is_ok() {
                                    if let Ok(core_speed) = storystream_core::PlaybackSpeed::new(new_speed) {
                                        current_db_state.speed = core_speed;
                                    }
                                }
                            }
                        }
                        Key::Char(']') => {
                            let current_speed = current_db_state.speed.value();
                            let new_speed = (current_speed + 0.1).min(3.0);
                            if let Ok(speed) = media_engine::PlaybackSpeed::new(new_speed) {
                                if engine.set_speed(speed).is_ok() {
                                    if let Ok(core_speed) = storystream_core::PlaybackSpeed::new(new_speed) {
                                        current_db_state.speed = core_speed;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => {}
            }
        }

        tokio::time::sleep(StdDuration::from_millis(100)).await;
    }

    Ok(())
}

fn draw_player_ui(
    term: &Term,
    engine: &AudioEngine,
    db_state: &storystream_core::PlaybackState,
    book: &Book,
) -> Result<()> {
    term.write_line(&format!("\n  {}", style(&book.title).bold().cyan()))
        .context("Failed to write title")?;

    if let Some(author) = &book.author {
        term.write_line(&format!("  by {}", style(author).dim()))
            .context("Failed to write author")?;
    }

    term.write_line("").context("Failed to write blank line")?;

    let position = if let Ok(pos) = engine.position() {
        core_duration_from_std(pos)
    } else {
        CoreDuration::from_millis(0)
    };

    let duration = book.duration;
    let progress = if duration.as_millis() > 0 {
        (position.as_millis() as f64 / duration.as_millis() as f64 * 100.0) as usize
    } else {
        0
    };

    term.write_line(&format!("  {} / {}", position.as_hms(), duration.as_hms()))
        .context("Failed to write position")?;

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

    let status = if let Ok(s) = engine.status() {
        match s {
            PlaybackStatus::Playing => style("Playing").green(),
            PlaybackStatus::Paused => style("Paused").yellow(),
            PlaybackStatus::Stopped => style("Stopped").red(),
        }
    } else {
        style("Unknown").dim()
    };

    term.write_line(&format!("  Status: {}", status))
        .context("Failed to write status")?;

    term.write_line(&format!("  Speed: {:.2}x", db_state.speed.value()))
        .context("Failed to write speed")?;

    term.write_line(&format!("  Volume: {}%", db_state.volume))
        .context("Failed to write volume")?;

    term.write_line("").context("Failed to write blank line")?;

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
        let engine = AudioEngine::new();
        assert!(engine.status().is_ok());
    }
}