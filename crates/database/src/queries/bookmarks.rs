//! Bookmark database operations

use crate::DbPool;
use storystream_core::{AppError, BookId, Bookmark, BookmarkId, Duration, Timestamp};

/// Creates a new bookmark
pub async fn create_bookmark(pool: &DbPool, bookmark: &Bookmark) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO bookmarks (id, book_id, position_ms, title, note, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(bookmark.id.as_string())
    .bind(bookmark.book_id.as_string())
    .bind(bookmark.position.as_millis() as i64)
    .bind(&bookmark.title)
    .bind(&bookmark.note)
    .bind(bookmark.created_at.as_millis())
    .bind(bookmark.updated_at.as_millis())
    .execute(pool)
    .await
    .map_err(|e| AppError::database("Failed to create bookmark", e))?;

    Ok(())
}

/// Gets a bookmark by ID
pub async fn get_bookmark(pool: &DbPool, id: BookmarkId) -> Result<Bookmark, AppError> {
    let row = sqlx::query(
        "SELECT id, book_id, position_ms, title, note, created_at, updated_at FROM bookmarks WHERE id = ?"
    )
        .bind(id.as_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database("Failed to fetch bookmark", e))?
        .ok_or_else(|| AppError::RecordNotFound {
            entity: "Bookmark".to_string(),
            identifier: id.to_string(),
        })?;

    row_to_bookmark(row)
}

/// Gets all bookmarks for a book
pub async fn get_book_bookmarks(pool: &DbPool, book_id: BookId) -> Result<Vec<Bookmark>, AppError> {
    let rows = sqlx::query(
        "SELECT id, book_id, position_ms, title, note, created_at, updated_at FROM bookmarks WHERE book_id = ? ORDER BY position_ms"
    )
        .bind(book_id.as_string())
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database("Failed to get book bookmarks", e))?;

    rows.into_iter().map(row_to_bookmark).collect()
}

/// Deletes a bookmark
pub async fn delete_bookmark(pool: &DbPool, id: BookmarkId) -> Result<(), AppError> {
    sqlx::query("DELETE FROM bookmarks WHERE id = ?")
        .bind(id.as_string())
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to delete bookmark", e))?;

    Ok(())
}

pub(crate) fn row_to_bookmark(row: sqlx::sqlite::SqliteRow) -> Result<Bookmark, AppError> {
    use sqlx::Row;

    let id_str: String = row
        .try_get("id")
        .map_err(|e| AppError::database("Missing bookmark ID", e))?;
    let id = BookmarkId::from_string(&id_str)
        .map_err(|e| AppError::database("Invalid bookmark ID", e))?;

    let book_id_str: String = row
        .try_get("book_id")
        .map_err(|e| AppError::database("Missing book ID", e))?;
    let book_id =
        BookId::from_string(&book_id_str).map_err(|e| AppError::database("Invalid book ID", e))?;

    let position_ms: i64 = row
        .try_get("position_ms")
        .map_err(|e| AppError::database("Missing position", e))?;
    let created_at_ms: i64 = row
        .try_get("created_at")
        .map_err(|e| AppError::database("Missing created_at", e))?;
    let updated_at_ms: i64 = row
        .try_get("updated_at")
        .map_err(|e| AppError::database("Missing updated_at", e))?;

    Ok(Bookmark {
        id,
        book_id,
        position: Duration::from_millis(position_ms as u64),
        title: row.try_get("title").ok(),
        note: row.try_get("note").ok(),
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
    use storystream_core::Book;

    async fn setup() -> DbPool {
        let pool = create_test_db().await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_get_bookmark() {
        let pool = setup().await;

        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        create_book(&pool, &book).await.unwrap();

        let bookmark = Bookmark::new(book.id, Duration::from_seconds(50));

        create_bookmark(&pool, &bookmark).await.unwrap();

        let retrieved = get_bookmark(&pool, bookmark.id).await.unwrap();
        assert_eq!(retrieved.position, Duration::from_seconds(50));
    }

    #[tokio::test]
    async fn test_get_book_bookmarks() {
        let pool = setup().await;

        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        create_book(&pool, &book).await.unwrap();

        let bookmark1 = Bookmark::new(book.id, Duration::from_seconds(25));
        let bookmark2 = Bookmark::new(book.id, Duration::from_seconds(75));

        create_bookmark(&pool, &bookmark1).await.unwrap();
        create_bookmark(&pool, &bookmark2).await.unwrap();

        let bookmarks = get_book_bookmarks(&pool, book.id).await.unwrap();
        assert_eq!(bookmarks.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_bookmark() {
        let pool = setup().await;

        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        create_book(&pool, &book).await.unwrap();

        let bookmark = Bookmark::new(book.id, Duration::from_seconds(50));

        create_bookmark(&pool, &bookmark).await.unwrap();
        delete_bookmark(&pool, bookmark.id).await.unwrap();

        let result = get_bookmark(&pool, bookmark.id).await;
        assert!(result.is_err());
    }
}
