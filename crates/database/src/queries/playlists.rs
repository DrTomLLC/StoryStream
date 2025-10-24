//! Playlist database operations

use crate::DbPool;
use storystream_core::{AppError, BookId, Playlist, PlaylistId, PlaylistItem, Timestamp};

/// Creates a new playlist
pub async fn create_playlist(pool: &DbPool, playlist: &Playlist) -> Result<(), AppError> {
    let playlist_type_str = match playlist.playlist_type {
        storystream_core::PlaylistType::Manual => "Manual",
        storystream_core::PlaylistType::Smart => "Smart",
    };

    let criteria_json = playlist
        .smart_criteria
        .as_ref()
        .map(|c| serde_json::to_string(c))
        .transpose()
        .map_err(|e| AppError::database("Failed to serialize criteria", e))?;

    sqlx::query(
        r#"
        INSERT INTO playlists (id, name, description, playlist_type, smart_criteria, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
        .bind(playlist.id.as_string())
        .bind(&playlist.name)
        .bind(&playlist.description)
        .bind(playlist_type_str)
        .bind(criteria_json)
        .bind(playlist.created_at.as_millis())
        .bind(playlist.updated_at.as_millis())
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to create playlist", e))?;

    Ok(())
}

/// Gets a playlist by ID
pub async fn get_playlist(pool: &DbPool, id: PlaylistId) -> Result<Playlist, AppError> {
    let row = sqlx::query(
        "SELECT id, name, description, playlist_type, smart_criteria, created_at, updated_at FROM playlists WHERE id = ?"
    )
        .bind(id.as_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database("Failed to fetch playlist", e))?
        .ok_or_else(|| AppError::RecordNotFound {
            entity: "Playlist".to_string(),
            identifier: id.to_string(),
        })?;

    row_to_playlist(row)
}

/// Deletes a playlist
pub async fn delete_playlist(pool: &DbPool, id: PlaylistId) -> Result<(), AppError> {
    sqlx::query("DELETE FROM playlists WHERE id = ?")
        .bind(id.as_string())
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to delete playlist", e))?;

    Ok(())
}

/// Adds a book to a playlist
pub async fn add_book_to_playlist(pool: &DbPool, item: &PlaylistItem) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO playlist_items (playlist_id, book_id, position, added_at) VALUES (?, ?, ?, ?)",
    )
    .bind(item.playlist_id.as_string())
    .bind(item.book_id.as_string())
    .bind(item.position as i64)
    .bind(item.added_at.as_millis())
    .execute(pool)
    .await
    .map_err(|e| AppError::database("Failed to add book to playlist", e))?;

    Ok(())
}

/// Removes a book from a playlist
pub async fn remove_book_from_playlist(
    pool: &DbPool,
    playlist_id: PlaylistId,
    book_id: BookId,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM playlist_items WHERE playlist_id = ? AND book_id = ?")
        .bind(playlist_id.as_string())
        .bind(book_id.as_string())
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to remove book from playlist", e))?;

    Ok(())
}

/// Gets all books in a playlist
pub async fn get_playlist_books(
    pool: &DbPool,
    playlist_id: PlaylistId,
) -> Result<Vec<storystream_core::Book>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT b.id, b.title, b.author, b.narrator, b.series, b.series_position,
               b.description, b.language, b.publisher, b.published_date, b.isbn,
               b.duration_ms, b.file_path, b.file_size, b.cover_art_path,
               b.added_date, b.last_played, b.play_count, b.is_favorite, b.rating, b.tags, b.deleted_at
        FROM books b
        JOIN playlist_items pi ON b.id = pi.book_id
        WHERE pi.playlist_id = ?
        ORDER BY pi.position
        "#,
    )
        .bind(playlist_id.as_string())
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database("Failed to get playlist books", e))?;

    rows.into_iter()
        .map(crate::queries::books::row_to_book)
        .collect()
}

fn row_to_playlist(row: sqlx::sqlite::SqliteRow) -> Result<Playlist, AppError> {
    use sqlx::Row;

    let id_str: String = row
        .try_get("id")
        .map_err(|e| AppError::database("Missing playlist ID", e))?;
    let id = PlaylistId::from_string(&id_str)
        .map_err(|e| AppError::database("Invalid playlist ID", e))?;

    let playlist_type_str: String = row
        .try_get("playlist_type")
        .map_err(|e| AppError::database("Missing playlist type", e))?;
    let playlist_type = match playlist_type_str.as_str() {
        "Manual" => storystream_core::PlaylistType::Manual,
        "Smart" => storystream_core::PlaylistType::Smart,
        _ => {
            return Err(AppError::InvalidArgument {
                argument: "playlist_type".to_string(),
                reason: "Invalid playlist type".to_string(),
            })
        }
    };

    let created_at_ms: i64 = row
        .try_get("created_at")
        .map_err(|e| AppError::database("Missing created_at", e))?;
    let updated_at_ms: i64 = row
        .try_get("updated_at")
        .map_err(|e| AppError::database("Missing updated_at", e))?;

    let criteria_json: Option<String> = row.try_get("smart_criteria").ok();
    let smart_criteria = criteria_json
        .filter(|s| !s.is_empty())
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|e| AppError::database("Failed to deserialize criteria", e))?;

    Ok(Playlist {
        id,
        name: row
            .try_get("name")
            .map_err(|e| AppError::database("Missing name", e))?,
        description: row.try_get("description").ok(),
        playlist_type,
        smart_criteria,
        created_at: Timestamp::from_millis(created_at_ms),
        updated_at: Timestamp::from_millis(updated_at_ms),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::create_test_db;
    use crate::migrations::run_migrations;
    use crate::queries::books::create_book;
    use std::path::PathBuf;
    use storystream_core::types::PlaylistItem;
    use storystream_core::{Book, Duration};

    async fn setup() -> DbPool {
        let pool = create_test_db().await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_get_playlist() {
        let pool = setup().await;

        let playlist = Playlist::new_manual("My Playlist".to_string());

        create_playlist(&pool, &playlist).await.unwrap();

        let retrieved = get_playlist(&pool, playlist.id).await.unwrap();
        assert_eq!(retrieved.name, "My Playlist");
    }

    #[tokio::test]
    async fn test_add_and_get_playlist_books() {
        let pool = setup().await;

        let playlist = Playlist::new_manual("My Playlist".to_string());
        create_playlist(&pool, &playlist).await.unwrap();

        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        create_book(&pool, &book).await.unwrap();

        let item = PlaylistItem::new(playlist.id, book.id, 0);
        add_book_to_playlist(&pool, &item).await.unwrap();

        let books = get_playlist_books(&pool, playlist.id).await.unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, book.id);
    }

    #[tokio::test]
    async fn test_remove_book_from_playlist() {
        let pool = setup().await;

        let playlist = Playlist::new_manual("My Playlist".to_string());
        create_playlist(&pool, &playlist).await.unwrap();

        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        create_book(&pool, &book).await.unwrap();

        let item = PlaylistItem::new(playlist.id, book.id, 0);
        add_book_to_playlist(&pool, &item).await.unwrap();

        remove_book_from_playlist(&pool, playlist.id, book.id)
            .await
            .unwrap();

        let books = get_playlist_books(&pool, playlist.id).await.unwrap();
        assert_eq!(books.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_playlist() {
        let pool = setup().await;

        let playlist = Playlist::new_manual("My Playlist".to_string());
        create_playlist(&pool, &playlist).await.unwrap();

        delete_playlist(&pool, playlist.id).await.unwrap();

        let result = get_playlist(&pool, playlist.id).await;
        assert!(result.is_err());
    }
}
