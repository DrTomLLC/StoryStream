//! Chapter database operations

use crate::DbPool;
use std::path::PathBuf;
use storystream_core::{AppError, BookId, Chapter, ChapterId, Duration};

/// Creates a new chapter
pub async fn create_chapter(pool: &DbPool, chapter: &Chapter) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO chapters (id, book_id, title, index_number, start_time_ms, end_time_ms, image_path)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
        .bind(chapter.id.as_string())
        .bind(chapter.book_id.as_string())
        .bind(&chapter.title)
        .bind(chapter.index as i64)
        .bind(chapter.start_time.as_millis() as i64)
        .bind(chapter.end_time.as_millis() as i64)
        .bind(chapter.image_path.as_ref().and_then(|p| p.to_str()))
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to create chapter", e))?;

    Ok(())
}

/// Gets a chapter by ID
pub async fn get_chapter(pool: &DbPool, id: ChapterId) -> Result<Chapter, AppError> {
    let row = sqlx::query(
        "SELECT id, book_id, title, index_number, start_time_ms, end_time_ms, image_path FROM chapters WHERE id = ?"
    )
        .bind(id.as_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database("Failed to fetch chapter", e))?
        .ok_or_else(|| AppError::RecordNotFound {
            entity: "Chapter".to_string(),
            identifier: id.to_string(),
        })?;

    row_to_chapter(row)
}

/// Gets all chapters for a book
pub async fn get_book_chapters(pool: &DbPool, book_id: BookId) -> Result<Vec<Chapter>, AppError> {
    let rows = sqlx::query(
        "SELECT id, book_id, title, index_number, start_time_ms, end_time_ms, image_path FROM chapters WHERE book_id = ? ORDER BY index_number"
    )
        .bind(book_id.as_string())
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database("Failed to get book chapters", e))?;

    rows.into_iter().map(row_to_chapter).collect()
}

/// Deletes a chapter
pub async fn delete_chapter(pool: &DbPool, id: ChapterId) -> Result<(), AppError> {
    sqlx::query("DELETE FROM chapters WHERE id = ?")
        .bind(id.as_string())
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to delete chapter", e))?;

    Ok(())
}

pub(crate) fn row_to_chapter(row: sqlx::sqlite::SqliteRow) -> Result<Chapter, AppError> {
    use sqlx::Row;

    let id_str: String = row
        .try_get("id")
        .map_err(|e| AppError::database("Missing chapter ID", e))?;
    let id =
        ChapterId::from_string(&id_str).map_err(|e| AppError::database("Invalid chapter ID", e))?;

    let book_id_str: String = row
        .try_get("book_id")
        .map_err(|e| AppError::database("Missing book ID", e))?;
    let book_id =
        BookId::from_string(&book_id_str).map_err(|e| AppError::database("Invalid book ID", e))?;

    let start_time_ms: i64 = row
        .try_get("start_time_ms")
        .map_err(|e| AppError::database("Missing start time", e))?;
    let end_time_ms: i64 = row
        .try_get("end_time_ms")
        .map_err(|e| AppError::database("Missing end time", e))?;
    let index: i64 = row
        .try_get("index_number")
        .map_err(|e| AppError::database("Missing index", e))?;

    let image_path_str: Option<String> = row.try_get("image_path").ok();

    Ok(Chapter {
        id,
        book_id,
        title: row
            .try_get("title")
            .map_err(|e| AppError::database("Missing title", e))?,
        index: index as u32,
        start_time: Duration::from_millis(start_time_ms as u64),
        end_time: Duration::from_millis(end_time_ms as u64),
        image_path: image_path_str.map(PathBuf::from),
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
    async fn test_create_and_get_chapter() {
        let pool = setup().await;

        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        create_book(&pool, &book).await.unwrap();

        let chapter = Chapter::new(
            book.id,
            "Chapter 1".to_string(),
            1,
            Duration::from_seconds(0),
            Duration::from_seconds(100),
        );

        create_chapter(&pool, &chapter).await.unwrap();

        let retrieved = get_chapter(&pool, chapter.id).await.unwrap();
        assert_eq!(retrieved.title, "Chapter 1");
    }

    #[tokio::test]
    async fn test_get_book_chapters() {
        let pool = setup().await;

        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(200),
        );
        create_book(&pool, &book).await.unwrap();

        let chapter1 = Chapter::new(
            book.id,
            "Chapter 1".to_string(),
            1,
            Duration::from_seconds(0),
            Duration::from_seconds(100),
        );
        let chapter2 = Chapter::new(
            book.id,
            "Chapter 2".to_string(),
            2,
            Duration::from_seconds(100),
            Duration::from_seconds(200),
        );

        create_chapter(&pool, &chapter1).await.unwrap();
        create_chapter(&pool, &chapter2).await.unwrap();

        let chapters = get_book_chapters(&pool, book.id).await.unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].index, 1);
        assert_eq!(chapters[1].index, 2);
    }

    #[tokio::test]
    async fn test_delete_chapter() {
        let pool = setup().await;

        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        create_book(&pool, &book).await.unwrap();

        let chapter = Chapter::new(
            book.id,
            "Chapter 1".to_string(),
            1,
            Duration::from_seconds(0),
            Duration::from_seconds(100),
        );

        create_chapter(&pool, &chapter).await.unwrap();
        delete_chapter(&pool, chapter.id).await.unwrap();

        let result = get_chapter(&pool, chapter.id).await;
        assert!(result.is_err());
    }
}
