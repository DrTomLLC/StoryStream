//! StoryStream Database Layer
//!
//! This crate provides database operations for the StoryStream audiobook player.
//! It uses SQLite with sqlx for type-safe database queries.

pub mod connection;
pub mod migrations;
pub mod queries;
pub mod search;

pub use connection::DbPool;
pub use migrations::{current_version, optimize, run_migrations, verify_integrity};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queries::books::{create_book, get_book, update_book};
    use crate::search::search_books;
    use connection::create_test_db;
    use std::path::PathBuf;
    use storystream_core::{AppError, Book, Duration, Timestamp};

    #[tokio::test]
    async fn test_database_migrations() -> Result<(), AppError> {
        let pool = create_test_db().await?;
        run_migrations(&pool).await?;

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations")
            .fetch_one(&pool)
            .await
            .map_err(|e| AppError::database("Failed to count migrations", e))?;

        assert!(count > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_full_database_workflow() -> Result<(), AppError> {
        let pool = create_test_db().await?;
        run_migrations(&pool).await?;

        let mut book = Book::new(
            "Test Workflow Book".to_string(),
            PathBuf::from("/test/workflow.mp3"),
            5_000_000,
            Duration::from_seconds(7200),
        );
        book.author = Some("Test Author".to_string());
        book.is_favorite = true;

        create_book(&pool, &book).await?;

        let retrieved = get_book(&pool, book.id).await?;
        assert_eq!(retrieved.title, "Test Workflow Book");
        assert_eq!(retrieved.author, Some("Test Author".to_string()));
        assert!(retrieved.is_favorite);

        Ok(())
    }

    #[tokio::test]
    async fn test_favorite_books() -> Result<(), AppError> {
        let pool = create_test_db().await?;
        run_migrations(&pool).await?;

        let mut book = Book::new(
            "Favorite Book".to_string(),
            PathBuf::from("/test/favorite.mp3"),
            1_000_000,
            Duration::from_seconds(3600),
        );
        book.is_favorite = true;

        create_book(&pool, &book).await?;

        let favorites = crate::queries::books::get_favorite_books(&pool).await?;
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].id, book.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_soft_delete() -> Result<(), AppError> {
        let pool = create_test_db().await?;
        run_migrations(&pool).await?;

        let mut book = Book::new(
            "To Delete".to_string(),
            PathBuf::from("/test/delete.mp3"),
            1_000_000,
            Duration::from_seconds(3600),
        );

        create_book(&pool, &book).await?;

        book.deleted_at = Some(Timestamp::now());
        update_book(&pool, &book).await?;

        let books = crate::queries::books::list_books(&pool).await?;
        assert_eq!(books.len(), 0);

        let retrieved = get_book(&pool, book.id).await?;
        assert!(retrieved.deleted_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_book_search() -> Result<(), AppError> {
        let pool = create_test_db().await?;
        run_migrations(&pool).await?;

        let mut book1 = Book::new(
            "The Great Gatsby".to_string(),
            PathBuf::from("/test/gatsby.mp3"),
            1_000_000,
            Duration::from_seconds(3600),
        );
        book1.author = Some("F. Scott Fitzgerald".to_string());

        let mut book2 = Book::new(
            "Great Expectations".to_string(),
            PathBuf::from("/test/expectations.mp3"),
            1_000_000,
            Duration::from_seconds(3600),
        );
        book2.author = Some("Charles Dickens".to_string());

        create_book(&pool, &book1).await?;
        create_book(&pool, &book2).await?;

        let results = search_books(&pool, "Great", 10).await?;
        assert_eq!(results.len(), 2);

        Ok(())
    }
}
