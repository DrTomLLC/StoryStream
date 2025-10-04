use crate::error::{LibraryError, Result};
use crate::import::{BookImporter, ImportOptions};
use crate::scanner::LibraryScanner;
use crate::LibraryConfig;
use std::path::Path;
use storystream_core::{Book, BookId, Duration};
use storystream_database::{
    connection::{connect, DatabaseConfig},
    migrations::run_migrations,
    queries::books,
    search::search_books,
    DbPool,
};
use tracing::info;

/// High-level library management
pub struct LibraryManager {
    pool: DbPool,
    #[allow(dead_code)]
    config: LibraryConfig,
    importer: BookImporter,
    scanner: Option<LibraryScanner>,
}

impl LibraryManager {
    /// Create a new library manager
    pub async fn new(config: LibraryConfig) -> Result<Self> {
        info!("Initializing library with database: {}", config.database_path);

        // Connect to database
        let db_config = DatabaseConfig::new(&config.database_path);
        let pool = connect(db_config).await?;

        // Run migrations
        run_migrations(&pool).await?;

        let importer = BookImporter::new(pool.clone());

        // Initialize scanner if watch directories configured
        let scanner = if !config.watch_directories.is_empty() {
            Some(LibraryScanner::new(config.watch_directories.clone()))
        } else {
            None
        };

        Ok(Self {
            pool,
            config,
            importer,
            scanner,
        })
    }

    /// Import a book from a file
    pub async fn import_book<P: AsRef<Path>>(
        &self,
        path: P,
        options: ImportOptions,
    ) -> Result<Book> {
        self.importer.import_file(path, options).await
    }

    /// Import multiple books
    pub async fn import_books<P: AsRef<Path>>(
        &self,
        paths: &[P],
        options: ImportOptions,
    ) -> Result<Vec<Book>> {
        self.importer.import_files(paths, options).await
    }

    /// Get all books in the library
    pub async fn list_books(&self) -> Result<Vec<Book>> {
        Ok(books::list_books(&self.pool).await?)
    }

    /// Get a specific book by ID
    pub async fn get_book(&self, id: BookId) -> Result<Book> {
        books::get_book(&self.pool, id)
            .await
            .map_err(|_| LibraryError::BookNotFound(id.to_string()))
    }

    /// Search for books
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<Book>> {
        let results = search_books(&self.pool, query, limit as i64).await?;
        Ok(results.into_iter().map(|r| r.item).collect())
    }

    /// Update a book
    pub async fn update_book(&self, book: &Book) -> Result<()> {
        Ok(books::update_book(&self.pool, book).await?)
    }

    /// Delete a book (hard delete)
    pub async fn delete_book(&self, id: BookId) -> Result<()> {
        Ok(books::delete_book(&self.pool, id).await?)
    }

    /// Soft delete a book
    pub async fn soft_delete_book(&self, id: BookId) -> Result<()> {
        let mut book = self.get_book(id).await?;
        book.delete();
        self.update_book(&book).await
    }

    /// Restore a soft-deleted book
    pub async fn restore_book(&self, id: BookId) -> Result<()> {
        let mut book = books::get_book(&self.pool, id).await?;
        book.restore();
        self.update_book(&book).await
    }

    /// Get favorite books
    pub async fn get_favorites(&self) -> Result<Vec<Book>> {
        Ok(books::get_favorite_books(&self.pool).await?)
    }

    /// Get recently played books
    pub async fn get_recently_played(&self, limit: usize) -> Result<Vec<Book>> {
        Ok(books::get_recently_played_books(&self.pool, limit as i64).await?)
    }

    /// Get books by author
    pub async fn get_by_author(&self, author: &str) -> Result<Vec<Book>> {
        Ok(books::get_books_by_author(&self.pool, author).await?)
    }

    /// Mark a book as played
    pub async fn mark_played(&self, id: BookId) -> Result<()> {
        let mut book = self.get_book(id).await?;
        book.mark_played();
        self.update_book(&book).await
    }

    /// Toggle favorite status
    pub async fn toggle_favorite(&self, id: BookId) -> Result<bool> {
        let mut book = self.get_book(id).await?;
        book.is_favorite = !book.is_favorite;
        self.update_book(&book).await?;
        Ok(book.is_favorite)
    }

    /// Set book rating
    pub async fn set_rating(&self, id: BookId, rating: Option<u8>) -> Result<()> {
        if let Some(r) = rating {
            if !(1..=5).contains(&r) {
                return Err(LibraryError::InvalidFile(
                    "Rating must be between 1 and 5".to_string(),
                ));
            }
        }

        let mut book = self.get_book(id).await?;
        book.rating = rating;
        self.update_book(&book).await
    }

    /// Get total library duration
    pub async fn total_duration(&self) -> Result<Duration> {
        let books = self.list_books().await?;
        let total_ms: u64 = books.iter().map(|b| b.duration.as_millis()).sum();
        Ok(Duration::from_millis(total_ms))
    }

    /// Get total library size in bytes
    pub async fn total_size(&self) -> Result<u64> {
        let books = self.list_books().await?;
        Ok(books.iter().map(|b| b.file_size).sum())
    }

    /// Get library statistics
    pub async fn get_stats(&self) -> Result<LibraryStats> {
        let books = self.list_books().await?;

        let total_books = books.len();
        let total_duration = Duration::from_millis(
            books.iter().map(|b| b.duration.as_millis()).sum()
        );
        let total_size = books.iter().map(|b| b.file_size).sum();
        let favorite_count = books.iter().filter(|b| b.is_favorite).count();
        let authors = books
            .iter()
            .filter_map(|b| b.author.as_ref())
            .collect::<std::collections::HashSet<_>>()
            .len();

        Ok(LibraryStats {
            total_books,
            total_duration,
            total_size,
            favorite_count,
            unique_authors: authors,
        })
    }

    /// Start watching directories for changes
    pub async fn start_watching(&mut self) -> Result<()> {
        if let Some(scanner) = &mut self.scanner {
            scanner.start().await?;
        }
        Ok(())
    }

    /// Stop watching directories
    pub async fn stop_watching(&mut self) -> Result<()> {
        if let Some(scanner) = &mut self.scanner {
            scanner.stop().await?;
        }
        Ok(())
    }

    /// Get database pool for advanced operations
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}

#[derive(Debug, Clone)]
pub struct LibraryStats {
    pub total_books: usize,
    pub total_duration: Duration,
    pub total_size: u64,
    pub favorite_count: usize,
    pub unique_authors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use storystream_database::migrations::run_migrations;
    use tempfile::NamedTempFile;

    async fn setup_test_manager() -> (LibraryManager, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let config = LibraryConfig::new(db_path);
        let manager = LibraryManager::new(config).await.unwrap();

        (manager, temp_file)
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let (_manager, _temp) = setup_test_manager().await;
    }

    #[tokio::test]
    async fn test_list_books_empty() {
        let (manager, _temp) = setup_test_manager().await;
        let books = manager.list_books().await.unwrap();
        assert_eq!(books.len(), 0);
    }

    #[tokio::test]
    async fn test_get_nonexistent_book() {
        let (manager, _temp) = setup_test_manager().await;
        let result = manager.get_book(BookId::new()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LibraryError::BookNotFound(_)));
    }

    #[tokio::test]
    async fn test_total_duration_empty() {
        let (manager, _temp) = setup_test_manager().await;
        let duration = manager.total_duration().await.unwrap();
        assert_eq!(duration.as_millis(), 0);
    }

    #[tokio::test]
    async fn test_total_size_empty() {
        let (manager, _temp) = setup_test_manager().await;
        let size = manager.total_size().await.unwrap();
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_library_stats_empty() {
        let (manager, _temp) = setup_test_manager().await;
        let stats = manager.get_stats().await.unwrap();

        assert_eq!(stats.total_books, 0);
        assert_eq!(stats.total_duration.as_millis(), 0);
        assert_eq!(stats.total_size, 0);
        assert_eq!(stats.favorite_count, 0);
        assert_eq!(stats.unique_authors, 0);
    }

    #[tokio::test]
    async fn test_set_rating_valid() {
        let (manager, _temp) = setup_test_manager().await;

        use std::path::PathBuf;
        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        books::create_book(manager.pool(), &book).await.unwrap();

        manager.set_rating(book.id, Some(4)).await.unwrap();
        let updated = manager.get_book(book.id).await.unwrap();
        assert_eq!(updated.rating, Some(4));
    }

    #[tokio::test]
    async fn test_set_rating_invalid() {
        let (manager, _temp) = setup_test_manager().await;

        use std::path::PathBuf;
        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        books::create_book(manager.pool(), &book).await.unwrap();

        let result = manager.set_rating(book.id, Some(10)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_toggle_favorite() {
        let (manager, _temp) = setup_test_manager().await;

        use std::path::PathBuf;
        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );

        books::create_book(manager.pool(), &book).await.unwrap();

        let is_fav = manager.toggle_favorite(book.id).await.unwrap();
        assert!(is_fav);

        let is_fav = manager.toggle_favorite(book.id).await.unwrap();
        assert!(!is_fav);
    }
}