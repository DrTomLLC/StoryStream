//! Playback state database operations

use crate::DbPool;
use storystream_core::{AppError, BookId, Duration, PlaybackSpeed, PlaybackState, Timestamp};

/// Creates or updates playback state for a book
pub async fn create_playback_state(pool: &DbPool, state: &PlaybackState) -> Result<(), AppError> {
    let equalizer_json = state
        .equalizer
        .as_ref()
        .map(|eq| serde_json::to_string(eq))
        .transpose()
        .map_err(|e| AppError::database("Failed to serialize equalizer", e))?;

    let sleep_timer_json = state
        .sleep_timer
        .as_ref()
        .map(|st| serde_json::to_string(st))
        .transpose()
        .map_err(|e| AppError::database("Failed to serialize sleep timer", e))?;

    sqlx::query(
        r#"
        INSERT INTO playback_state (
            book_id, position_ms, speed, pitch_correction, volume, is_playing,
            equalizer_preset, sleep_timer, skip_silence, volume_boost, last_updated
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(book_id) DO UPDATE SET
            position_ms = excluded.position_ms,
            speed = excluded.speed,
            pitch_correction = excluded.pitch_correction,
            volume = excluded.volume,
            is_playing = excluded.is_playing,
            equalizer_preset = excluded.equalizer_preset,
            sleep_timer = excluded.sleep_timer,
            skip_silence = excluded.skip_silence,
            volume_boost = excluded.volume_boost,
            last_updated = excluded.last_updated
        "#,
    )
        .bind(state.book_id.as_string())
        .bind(state.position.as_millis() as i64)
        .bind(state.speed.value() as f64)
        .bind(state.speed.has_pitch_correction() as i64)
        .bind(state.volume as i64)
        .bind(state.is_playing as i64)
        .bind(equalizer_json)
        .bind(sleep_timer_json)
        .bind(state.skip_silence as i64)
        .bind(state.volume_boost as i64)
        .bind(state.last_updated.as_millis())
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to save playback state", e))?;

    Ok(())
}

/// Gets playback state for a book
pub async fn get_playback_state(pool: &DbPool, book_id: BookId) -> Result<PlaybackState, AppError> {
    let row = sqlx::query(
        r#"
        SELECT book_id, position_ms, speed, pitch_correction, volume, is_playing,
               equalizer_preset, sleep_timer, skip_silence, volume_boost, last_updated
        FROM playback_state WHERE book_id = ?
        "#,
    )
        .bind(book_id.as_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database("Failed to fetch playback state", e))?
        .ok_or_else(|| AppError::RecordNotFound {
            entity: "PlaybackState".to_string(),
            identifier: book_id.to_string(),
        })?;

    row_to_playback_state(row)
}

/// Updates playback position (for frequent saves)
pub async fn update_playback_state(pool: &DbPool, book_id: BookId, position: Duration) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE playback_state SET position_ms = ?, last_updated = ? WHERE book_id = ?"
    )
        .bind(position.as_millis() as i64)
        .bind(Timestamp::now().as_millis())
        .bind(book_id.as_string())
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to update playback position", e))?;

    Ok(())
}

fn row_to_playback_state(row: sqlx::sqlite::SqliteRow) -> Result<PlaybackState, AppError> {
    use sqlx::Row;

    let book_id_str: String = row.try_get("book_id")
        .map_err(|e| AppError::database("Missing book ID", e))?;
    let book_id = BookId::from_string(&book_id_str)
        .map_err(|e| AppError::database("Invalid book ID", e))?;

    let position_ms: i64 = row.try_get("position_ms")
        .map_err(|e| AppError::database("Missing position", e))?;
    let speed: f64 = row.try_get("speed")
        .map_err(|e| AppError::database("Missing speed", e))?;
    let pitch_correction: i64 = row.try_get("pitch_correction")
        .map_err(|e| AppError::database("Missing pitch correction", e))?;
    let volume: i64 = row.try_get("volume")
        .map_err(|e| AppError::database("Missing volume", e))?;
    let is_playing: i64 = row.try_get("is_playing")
        .map_err(|e| AppError::database("Missing is_playing", e))?;
    let skip_silence: i64 = row.try_get("skip_silence")
        .map_err(|e| AppError::database("Missing skip_silence", e))?;
    let volume_boost: i64 = row.try_get("volume_boost")
        .map_err(|e| AppError::database("Missing volume_boost", e))?;
    let last_updated_ms: i64 = row.try_get("last_updated")
        .map_err(|e| AppError::database("Missing last_updated", e))?;

    let equalizer_json: Option<String> = row.try_get("equalizer_preset").ok();
    let equalizer = equalizer_json
        .filter(|s| !s.is_empty())
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|e| AppError::database("Failed to deserialize equalizer", e))?;

    let sleep_timer_json: Option<String> = row.try_get("sleep_timer").ok();
    let sleep_timer = sleep_timer_json
        .filter(|s| !s.is_empty())
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|e| AppError::database("Failed to deserialize sleep timer", e))?;

    let speed_obj = PlaybackSpeed::new_unchecked(speed as f32)
        .with_pitch_correction(pitch_correction != 0);

    Ok(PlaybackState {
        book_id,
        position: Duration::from_millis(position_ms as u64),
        speed: speed_obj,
        volume: volume as u8,
        is_playing: is_playing != 0,
        equalizer,
        sleep_timer,
        skip_silence: skip_silence != 0,
        volume_boost: volume_boost as u8,
        last_updated: Timestamp::from_millis(last_updated_ms),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::create_test_db;
    use crate::migrations::run_migrations;
    use crate::queries::books::create_book;
    use storystream_core::Book;
    use std::path::PathBuf;

    async fn setup() -> DbPool {
        let pool = create_test_db().await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_get_playback_state() {
        let pool = setup().await;

        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        create_book(&pool, &book).await.unwrap();

        let state = PlaybackState::new(book.id);

        create_playback_state(&pool, &state).await.unwrap();

        let retrieved = get_playback_state(&pool, book.id).await.unwrap();
        assert_eq!(retrieved.book_id, book.id);
        assert_eq!(retrieved.position, Duration::from_millis(0));
    }

    #[tokio::test]
    async fn test_update_playback_state() {
        let pool = setup().await;

        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        create_book(&pool, &book).await.unwrap();

        let state = PlaybackState::new(book.id);
        create_playback_state(&pool, &state).await.unwrap();

        update_playback_state(&pool, book.id, Duration::from_seconds(50))
            .await
            .unwrap();

        let retrieved = get_playback_state(&pool, book.id).await.unwrap();
        assert_eq!(retrieved.position, Duration::from_seconds(50));
    }
}