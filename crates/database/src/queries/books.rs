//! Book database operations

use crate::DbPool;
use std::path::PathBuf;
use storystream_core::{AppError, Book, BookId, Duration, Timestamp};

/// Creates a new book in the database
pub async fn create_book(pool: &DbPool, book: &Book) -> Result<(), AppError> {
    let tags_json = serde_json::to_string(&book.tags)
        .map_err(|e| AppError::database("Failed to serialize tags", e))?;

    sqlx::query(
        r#"
        INSERT INTO books (
            id, title, author, narrator, series, series_position,
            description, language, publisher, published_date, isbn,
            duration_ms, file_path, file_size, cover_art_path,
            added_date, last_played, play_count, is_favorite, rating, tags, deleted_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(book.id.as_string())
    .bind(&book.title)
    .bind(&book.author)
    .bind(&book.narrator)
    .bind(&book.series)
    .bind(book.series_position)
    .bind(&book.description)
    .bind(&book.language)
    .bind(&book.publisher)
    .bind(&book.published_date)
    .bind(&book.isbn)
    .bind(book.duration.as_millis() as i64)
    .bind(book.file_path.to_str())
    .bind(book.file_size as i64)
    .bind(book.cover_art_path.as_ref().and_then(|p| p.to_str()))
    .bind(book.added_date.as_millis())
    .bind(book.last_played.map(|t| t.as_millis()))
    .bind(book.play_count as i64)
    .bind(book.is_favorite as i64)
    .bind(book.rating.map(|r| r as i64))
    .bind(tags_json)
    .bind(book.deleted_at.map(|t| t.as_millis()))
    .execute(pool)
    .await
    .map_err(|e| AppError::database("Failed to create book", e))?;

    Ok(())
}

/// Gets a book by ID
pub async fn get_book(pool: &DbPool, id: BookId) -> Result<Book, AppError> {
    let row = sqlx::query(
        r#"
        SELECT id, title, author, narrator, series, series_position,
               description, language, publisher, published_date, isbn,
               duration_ms, file_path, file_size, cover_art_path,
               added_date, last_played, play_count, is_favorite, rating, tags, deleted_at
        FROM books WHERE id = ?
        "#,
    )
    .bind(id.as_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::database("Failed to fetch book", e))?
    .ok_or_else(|| AppError::RecordNotFound {
        entity: "Book".to_string(),
        identifier: id.to_string(),
    })?;

    row_to_book(row)
}

/// Updates an existing book
pub async fn update_book(pool: &DbPool, book: &Book) -> Result<(), AppError> {
    let tags_json = serde_json::to_string(&book.tags)
        .map_err(|e| AppError::database("Failed to serialize tags", e))?;

    sqlx::query(
        r#"
        UPDATE books SET
            title = ?, author = ?, narrator = ?, series = ?, series_position = ?,
            description = ?, language = ?, publisher = ?, published_date = ?, isbn = ?,
            duration_ms = ?, file_path = ?, file_size = ?, cover_art_path = ?,
            last_played = ?, play_count = ?, is_favorite = ?, rating = ?, tags = ?, deleted_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&book.title)
    .bind(&book.author)
    .bind(&book.narrator)
    .bind(&book.series)
    .bind(book.series_position)
    .bind(&book.description)
    .bind(&book.language)
    .bind(&book.publisher)
    .bind(&book.published_date)
    .bind(&book.isbn)
    .bind(book.duration.as_millis() as i64)
    .bind(book.file_path.to_str())
    .bind(book.file_size as i64)
    .bind(book.cover_art_path.as_ref().and_then(|p| p.to_str()))
    .bind(book.last_played.map(|t| t.as_millis()))
    .bind(book.play_count as i64)
    .bind(book.is_favorite as i64)
    .bind(book.rating.map(|r| r as i64))
    .bind(tags_json)
    .bind(book.deleted_at.map(|t| t.as_millis()))
    .bind(book.id.as_string())
    .execute(pool)
    .await
    .map_err(|e| AppError::database("Failed to update book", e))?;

    Ok(())
}

/// Deletes a book (hard delete)
pub async fn delete_book(pool: &DbPool, id: BookId) -> Result<(), AppError> {
    sqlx::query("DELETE FROM books WHERE id = ?")
        .bind(id.as_string())
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to delete book", e))?;

    Ok(())
}

/// Lists all books (excluding soft-deleted)
pub async fn list_books(pool: &DbPool) -> Result<Vec<Book>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, author, narrator, series, series_position,
               description, language, publisher, published_date, isbn,
               duration_ms, file_path, file_size, cover_art_path,
               added_date, last_played, play_count, is_favorite, rating, tags, deleted_at
        FROM books
        WHERE deleted_at IS NULL
        ORDER BY added_date DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database("Failed to list books", e))?;

    rows.into_iter().map(row_to_book).collect()
}

/// Gets books by author
pub async fn get_books_by_author(pool: &DbPool, author: &str) -> Result<Vec<Book>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, author, narrator, series, series_position,
               description, language, publisher, published_date, isbn,
               duration_ms, file_path, file_size, cover_art_path,
               added_date, last_played, play_count, is_favorite, rating, tags, deleted_at
        FROM books
        WHERE author = ? AND deleted_at IS NULL
        ORDER BY title
        "#,
    )
    .bind(author)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database("Failed to get books by author", e))?;

    rows.into_iter().map(row_to_book).collect()
}

/// Gets favorite books
pub async fn get_favorite_books(pool: &DbPool) -> Result<Vec<Book>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, author, narrator, series, series_position,
               description, language, publisher, published_date, isbn,
               duration_ms, file_path, file_size, cover_art_path,
               added_date, last_played, play_count, is_favorite, rating, tags, deleted_at
        FROM books
        WHERE is_favorite = 1 AND deleted_at IS NULL
        ORDER BY title
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database("Failed to get favorite books", e))?;

    rows.into_iter().map(row_to_book).collect()
}

/// Gets recently played books
pub async fn get_recently_played_books(pool: &DbPool, limit: i64) -> Result<Vec<Book>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, author, narrator, series, series_position,
               description, language, publisher, published_date, isbn,
               duration_ms, file_path, file_size, cover_art_path,
               added_date, last_played, play_count, is_favorite, rating, tags, deleted_at
        FROM books
        WHERE last_played IS NOT NULL AND deleted_at IS NULL
        ORDER BY last_played DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database("Failed to get recently played books", e))?;

    rows.into_iter().map(row_to_book).collect()
}

/// Converts a database row to a Book
pub(crate) fn row_to_book(row: sqlx::sqlite::SqliteRow) -> Result<Book, AppError> {
    use sqlx::Row;

    let id_str: String = row
        .try_get("id")
        .map_err(|e| AppError::database("Missing book ID", e))?;
    let id = BookId::from_string(&id_str).map_err(|e| AppError::database("Invalid book ID", e))?;

    let tags_json: String = row
        .try_get("tags")
        .map_err(|e| AppError::database("Missing tags", e))?;
    let tags: Vec<String> = serde_json::from_str(&tags_json)
        .map_err(|e| AppError::database("Failed to deserialize tags", e))?;

    let duration_ms: i64 = row
        .try_get("duration_ms")
        .map_err(|e| AppError::database("Missing duration", e))?;

    let file_path_str: String = row
        .try_get("file_path")
        .map_err(|e| AppError::database("Missing file path", e))?;

    let added_date_ms: i64 = row
        .try_get("added_date")
        .map_err(|e| AppError::database("Missing added date", e))?;

    let last_played_ms: Option<i64> = row.try_get("last_played").ok();
    let play_count: i64 = row
        .try_get("play_count")
        .map_err(|e| AppError::database("Missing play count", e))?;
    let is_favorite: i64 = row
        .try_get("is_favorite")
        .map_err(|e| AppError::database("Missing is_favorite", e))?;
    let rating: Option<i64> = row.try_get("rating").ok();
    let file_size: i64 = row
        .try_get("file_size")
        .map_err(|e| AppError::database("Missing file size", e))?;
    let deleted_at_ms: Option<i64> = row.try_get("deleted_at").ok();

    let cover_art_path_str: Option<String> = row.try_get("cover_art_path").ok();

    Ok(Book {
        id,
        title: row
            .try_get("title")
            .map_err(|e| AppError::database("Missing title", e))?,
        author: row.try_get("author").ok(),
        narrator: row.try_get("narrator").ok(),
        series: row.try_get("series").ok(),
        series_position: row.try_get("series_position").ok(),
        description: row.try_get("description").ok(),
        language: row.try_get("language").ok(),
        publisher: row.try_get("publisher").ok(),
        published_date: row.try_get("published_date").ok(),
        isbn: row.try_get("isbn").ok(),
        duration: Duration::from_millis(duration_ms as u64),
        file_path: PathBuf::from(file_path_str),
        file_size: file_size as u64,
        cover_art_path: cover_art_path_str.map(PathBuf::from),
        added_date: Timestamp::from_millis(added_date_ms),
        last_played: last_played_ms.map(Timestamp::from_millis),
        play_count: play_count as u32,
        is_favorite: is_favorite != 0,
        rating: rating.filter(|&r| r >= 1 && r <= 5).map(|r| r as u8),
        tags,
        deleted_at: deleted_at_ms.map(Timestamp::from_millis),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::create_test_db;
    use crate::migrations::run_migrations;

    async fn setup() -> Result<DbPool, AppError> {
        let pool = create_test_db().await?;
        run_migrations(&pool).await?;
        Ok(pool)
    }

    fn create_test_book_with_path(path: &str) -> Book {
        Book::new(
            "Test Book".to_string(),
            PathBuf::from(path),
            1_000_000,
            Duration::from_seconds(3600),
        )
    }

    #[tokio::test]
    async fn test_create_and_get_book() {
        let pool = setup().await.expect("Failed to setup database");
        let book = create_test_book_with_path("/test/create_get.mp3");

        create_book(&pool, &book)
            .await
            .expect("Failed to create book");

        let retrieved = get_book(&pool, book.id).await.expect("Failed to get book");
        assert_eq!(retrieved.id, book.id);
        assert_eq!(retrieved.title, book.title);
    }

    #[tokio::test]
    async fn test_update_book() {
        let pool = setup().await.expect("Failed to setup database");
        let mut book = create_test_book_with_path("/test/update.mp3");

        create_book(&pool, &book)
            .await
            .expect("Failed to create book");

        book.title = "Updated Title".to_string();
        book.is_favorite = true;
        update_book(&pool, &book)
            .await
            .expect("Failed to update book");

        let retrieved = get_book(&pool, book.id)
            .await
            .expect("Failed to get updated book");
        assert_eq!(retrieved.title, "Updated Title");
        assert!(retrieved.is_favorite);
    }

    #[tokio::test]
    async fn test_delete_book() {
        let pool = setup().await.expect("Failed to setup database");
        let book = create_test_book_with_path("/test/delete.mp3");

        create_book(&pool, &book)
            .await
            .expect("Failed to create book");
        delete_book(&pool, book.id)
            .await
            .expect("Failed to delete book");

        let result = get_book(&pool, book.id).await;
        assert!(result.is_err(), "Book should not exist after deletion");
    }

    #[tokio::test]
    async fn test_list_books() {
        let pool = setup().await.expect("Failed to setup database");

        let book1 = create_test_book_with_path("/test/list_1.mp3");
        let mut book2 = create_test_book_with_path("/test/list_2.mp3");
        book2.title = "Second Book".to_string();

        create_book(&pool, &book1)
            .await
            .expect("Failed to create book 1");
        create_book(&pool, &book2)
            .await
            .expect("Failed to create book 2");

        let books = list_books(&pool).await.expect("Failed to list books");
        assert_eq!(books.len(), 2);
    }

    #[tokio::test]
    async fn test_get_books_by_author() {
        let pool = setup().await.expect("Failed to setup database");

        let mut book1 = create_test_book_with_path("/test/author_1.mp3");
        book1.title = "Book 1".to_string();
        book1.author = Some("Author A".to_string());

        let mut book2 = create_test_book_with_path("/test/author_2.mp3");
        book2.title = "Book 2".to_string();
        book2.author = Some("Author A".to_string());

        create_book(&pool, &book1)
            .await
            .expect("Failed to create book 1");
        create_book(&pool, &book2)
            .await
            .expect("Failed to create book 2");

        let books = get_books_by_author(&pool, "Author A")
            .await
            .expect("Failed to get books by author");
        assert_eq!(books.len(), 2);
    }

    #[tokio::test]
    async fn test_get_favorite_books() {
        let pool = setup().await.expect("Failed to setup database");

        let mut book1 = create_test_book_with_path("/test/fav_1.mp3");
        book1.is_favorite = true;

        let book2 = create_test_book_with_path("/test/fav_2.mp3");

        create_book(&pool, &book1)
            .await
            .expect("Failed to create favorite book");
        create_book(&pool, &book2)
            .await
            .expect("Failed to create regular book");

        let favorites = get_favorite_books(&pool)
            .await
            .expect("Failed to get favorite books");
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].id, book1.id);
    }

    #[tokio::test]
    async fn test_get_recently_played_books() {
        let pool = setup().await.expect("Failed to setup database");

        let mut book1 = create_test_book_with_path("/test/recent_1.mp3");
        book1.last_played = Some(Timestamp::now());

        let book2 = create_test_book_with_path("/test/recent_2.mp3");

        create_book(&pool, &book1)
            .await
            .expect("Failed to create played book");
        create_book(&pool, &book2)
            .await
            .expect("Failed to create unplayed book");

        let recent = get_recently_played_books(&pool, 10)
            .await
            .expect("Failed to get recently played books");
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].id, book1.id);
    }
}
