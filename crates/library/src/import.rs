use crate::error::{LibraryError, Result};
use crate::metadata::AudioMetadataExtractor;
use std::path::{Path, PathBuf};
use storystream_core::Book;
use storystream_database::{queries::books, DbPool};
use tracing::{info, warn};

/// Options for importing books
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Override title from metadata
    pub title: Option<String>,
    /// Override author from metadata
    pub author: Option<String>,
    /// Override narrator from metadata
    pub narrator: Option<String>,
    /// Extract cover art
    pub extract_cover: bool,
    /// Cover art output directory
    pub cover_output_dir: Option<PathBuf>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            narrator: None,
            extract_cover: true,
            cover_output_dir: None,
        }
    }
}

/// Handles book import operations
pub struct BookImporter {
    pool: DbPool,
}

impl BookImporter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Import a single audio file as a book
    pub async fn import_file<P: AsRef<Path>>(
        &self,
        path: P,
        options: ImportOptions,
    ) -> Result<Book> {
        let path = path.as_ref();

        info!("Importing book from: {}", path.display());

        // Verify file exists and is supported
        if !path.exists() {
            return Err(LibraryError::FileNotFound(path.display().to_string()));
        }

        if !AudioMetadataExtractor::is_supported(path) {
            return Err(LibraryError::UnsupportedFormat(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            ));
        }

        // Extract metadata
        let metadata = AudioMetadataExtractor::extract(path).map_err(|e| {
            LibraryError::ImportFailed(format!("Metadata extraction failed: {}", e))
        })?;

        // Determine title
        let title = options
            .title
            .or(metadata.title)
            .or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .ok_or_else(|| LibraryError::ImportFailed("Could not determine title".to_string()))?;

        // Create book
        let mut book = Book::new(
            title,
            path.to_path_buf(),
            metadata.file_size,
            metadata.duration,
        );

        // Set metadata
        book.author = options.author.or(metadata.artist);
        book.narrator = options.narrator;

        // Handle cover art
        if options.extract_cover {
            if let Some(cover_data) = metadata.cover_art {
                if let Some(output_dir) = options.cover_output_dir {
                    match self.save_cover_art(&book, &cover_data, &output_dir).await {
                        Ok(cover_path) => book.cover_art_path = Some(cover_path),
                        Err(e) => warn!("Failed to save cover art: {}", e),
                    }
                }
            }
        }

        // Save to database
        books::create_book(&self.pool, &book)
            .await
            .map_err(|e| LibraryError::ImportFailed(format!("Database error: {}", e)))?;

        info!("Successfully imported book: {} ({})", book.title, book.id);
        Ok(book)
    }

    /// Import multiple files
    pub async fn import_files<P: AsRef<Path>>(
        &self,
        paths: &[P],
        options: ImportOptions,
    ) -> Result<Vec<Book>> {
        let mut books = Vec::new();
        let mut errors = Vec::new();

        for path in paths {
            match self.import_file(path, options.clone()).await {
                Ok(book) => books.push(book),
                Err(e) => {
                    let path_str = path.as_ref().display().to_string();
                    warn!("Failed to import {}: {}", path_str, e);
                    errors.push((path_str, e));
                }
            }
        }

        if !errors.is_empty() && books.is_empty() {
            return Err(LibraryError::ImportFailed(format!(
                "Failed to import {} files",
                errors.len()
            )));
        }

        Ok(books)
    }

    async fn save_cover_art(
        &self,
        book: &Book,
        cover_data: &[u8],
        output_dir: &Path,
    ) -> Result<PathBuf> {
        tokio::fs::create_dir_all(output_dir).await?;

        let filename = format!("{}.jpg", book.id);
        let cover_path = output_dir.join(filename);

        tokio::fs::write(&cover_path, cover_data).await?;

        Ok(cover_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use storystream_database::{
        connection::{connect, DatabaseConfig},
        migrations::run_migrations,
    };
    use tempfile::NamedTempFile;

    async fn setup_test_db() -> (DbPool, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let config = DatabaseConfig::new(db_path);
        let pool = connect(config).await.unwrap();
        run_migrations(&pool).await.unwrap();

        (pool, temp_file)
    }

    #[test]
    fn test_import_options_default() {
        let options = ImportOptions::default();
        assert!(options.title.is_none());
        assert!(options.author.is_none());
        assert!(options.extract_cover);
    }

    #[tokio::test]
    async fn test_importer_creation() {
        let (pool, _temp) = setup_test_db().await;
        let _importer = BookImporter::new(pool);
    }

    #[tokio::test]
    async fn test_import_nonexistent_file() {
        let (pool, _temp) = setup_test_db().await;
        let importer = BookImporter::new(pool);

        let result = importer
            .import_file("/nonexistent/file.mp3", ImportOptions::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LibraryError::FileNotFound(_)));
    }

    #[tokio::test]
    async fn test_import_unsupported_format() {
        let (pool, _temp) = setup_test_db().await;
        let importer = BookImporter::new(pool);

        // Create a temp file with unsupported extension
        let temp_file = NamedTempFile::with_suffix(".txt").unwrap();
        std::fs::write(temp_file.path(), b"not audio").unwrap();

        let result = importer
            .import_file(temp_file.path(), ImportOptions::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LibraryError::UnsupportedFormat(_)
        ));
    }
}