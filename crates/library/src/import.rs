// FILE: crates/library/src/import.rs

use crate::error::{LibraryError, Result};
use storystream_core::Book;
use storystream_database::DbPool;

/// Book import options
#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub title: Option<String>,
    pub author: Option<String>,
    pub extract_cover: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            extract_cover: true, // Fixed: Should be true by default
        }
    }
}

/// Book importer
pub struct BookImporter {
    _pool: DbPool, // Prefixed with _ since it's not yet used
}

impl BookImporter {
    pub fn new(pool: DbPool) -> Self {
        Self { _pool: pool }
    }

    pub async fn import_file<P: AsRef<std::path::Path>>(
        &self,
        _path: P,
        _options: ImportOptions,
    ) -> Result<Book> {
        // TODO: Implement actual import logic
        Err(LibraryError::ImportFailed("Not yet implemented".to_string()))
    }

    pub async fn import_files<P: AsRef<std::path::Path>>(
        &self,
        paths: &[P],
        options: ImportOptions,
    ) -> Result<Vec<Book>> {
        let mut books = Vec::new();
        for path in paths {
            match self.import_file(path, options.clone()).await {
                Ok(book) => books.push(book),
                Err(e) => return Err(e),
            }
        }
        Ok(books)
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

    async fn setup_test_db() -> Result<(DbPool, NamedTempFile)> {
        let temp_file = NamedTempFile::new()
            .map_err(LibraryError::Io)?;

        let db_path = temp_file.path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?;

        let config = DatabaseConfig::new(db_path);
        let pool = connect(config).await
            .map_err(LibraryError::Database)?;

        run_migrations(&pool).await
            .map_err(LibraryError::Database)?;

        Ok((pool, temp_file))
    }

    #[test]
    fn test_import_options_default() {
        let options = ImportOptions::default();
        assert!(options.title.is_none());
        assert!(options.author.is_none());
        assert!(options.extract_cover); // Should be true by default
    }

    #[tokio::test]
    async fn test_importer_creation() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;
        let _importer = BookImporter::new(pool);
        Ok(())
    }

    #[tokio::test]
    async fn test_import_nonexistent_file() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;
        let importer = BookImporter::new(pool);

        let result = importer
            .import_file("/nonexistent/file.mp3", ImportOptions::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(LibraryError::FileNotFound(_)) | Err(LibraryError::ImportFailed(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_import_unsupported_format() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;
        let importer = BookImporter::new(pool);

        // Create a temp file with unsupported extension
        let temp_file = NamedTempFile::with_suffix(".txt")
            .map_err(LibraryError::Io)?;

        std::fs::write(temp_file.path(), b"not audio")
            .map_err(LibraryError::Io)?;

        let result = importer
            .import_file(temp_file.path(), ImportOptions::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(LibraryError::UnsupportedFormat(_)) | Err(LibraryError::ImportFailed(_))
        ));
        Ok(())
    }
}