//! Full-text search using FTS5

use crate::DbPool;
use storystream_core::{AppError, Book, Bookmark, Chapter};

/// Search result with relevance ranking
#[derive(Debug, Clone)]
pub struct SearchResult<T> {
    pub item: T,
    pub rank: f64,
}

/// Searches books by text query
pub async fn search_books(
    pool: &DbPool,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult<Book>>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT b.id, b.title, b.author, b.narrator, b.series, b.series_position,
               b.description, b.language, b.publisher, b.published_date, b.isbn,
               b.duration_ms, b.file_path, b.file_size, b.cover_art_path,
               b.added_date, b.last_played, b.play_count, b.is_favorite, b.rating, b.tags, b.deleted_at,
               bm.rank as rank
        FROM books_fts bm
        JOIN books b ON bm.rowid = b.rowid
        WHERE books_fts MATCH ?
        AND b.deleted_at IS NULL
        ORDER BY rank
        LIMIT ?
        "#,
    )
        .bind(query)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database("Failed to search books", e))?;

    rows.into_iter()
        .map(|row| {
            use sqlx::Row;
            let rank: f64 = row.try_get("rank").unwrap_or(0.0);
            let book = crate::queries::books::row_to_book(row)?;
            Ok(SearchResult { item: book, rank })
        })
        .collect()
}

/// Searches chapters by text query
pub async fn search_chapters(
    pool: &DbPool,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult<Chapter>>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT c.id, c.book_id, c.title, c.index_number, c.start_time_ms, c.end_time_ms, c.image_path,
               cm.rank as rank
        FROM chapters_fts cm
        JOIN chapters c ON cm.rowid = c.rowid
        WHERE chapters_fts MATCH ?
        ORDER BY rank
        LIMIT ?
        "#,
    )
        .bind(query)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database("Failed to search chapters", e))?;

    rows.into_iter()
        .map(|row| {
            use sqlx::Row;
            let rank: f64 = row.try_get("rank").unwrap_or(0.0);
            let chapter = crate::queries::chapters::row_to_chapter(row)?;
            Ok(SearchResult {
                item: chapter,
                rank,
            })
        })
        .collect()
}

/// Searches bookmarks by text query
pub async fn search_bookmarks(
    pool: &DbPool,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult<Bookmark>>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT bm.id, bm.book_id, bm.position_ms, bm.title, bm.note, bm.created_at, bm.updated_at,
               bmf.rank as rank
        FROM bookmarks_fts bmf
        JOIN bookmarks bm ON bmf.rowid = bm.rowid
        WHERE bookmarks_fts MATCH ?
        ORDER BY rank
        LIMIT ?
        "#,
    )
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database("Failed to search bookmarks", e))?;

    rows.into_iter()
        .map(|row| {
            use sqlx::Row;
            let rank: f64 = row.try_get("rank").unwrap_or(0.0);
            let bookmark = crate::queries::bookmarks::row_to_bookmark(row)?;
            Ok(SearchResult {
                item: bookmark,
                rank,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::create_test_db;
    use crate::migrations::run_migrations;
    use crate::queries::books::create_book;
    use std::path::PathBuf;
    use storystream_core::{Book, Duration};

    async fn setup() -> DbPool {
        let pool = create_test_db().await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_search_books() {
        let pool = setup().await;

        let mut book1 = Book::new(
            "The Great Adventure".to_string(),
            PathBuf::from("/test1.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        book1.author = Some("John Smith".to_string());

        let mut book2 = Book::new(
            "Another Story".to_string(),
            PathBuf::from("/test2.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        book2.author = Some("Jane Doe".to_string());

        create_book(&pool, &book1).await.unwrap();
        create_book(&pool, &book2).await.unwrap();

        let results = search_books(&pool, "Adventure", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item.title, "The Great Adventure");
    }
}
