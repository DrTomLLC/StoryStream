// FILE: crates/library/src/import.rs

use crate::error::{LibraryError, Result};
use crate::metadata::{ExtractedMetadata, MetadataExtractor};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use storystream_core::Book;
use storystream_database::{queries::books, DbPool};

/// Book import options
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Override title from metadata
    pub title: Option<String>,
    /// Override author from metadata
    pub author: Option<String>,
    /// Whether to extract cover art
    pub extract_cover: bool,
    /// Whether to overwrite if book already exists (based on path)
    pub overwrite_existing: bool,
    /// Whether to skip files with errors instead of failing the whole import
    pub skip_on_error: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            extract_cover: true,
            overwrite_existing: false,
            skip_on_error: false,
        }
    }
}

impl ImportOptions {
    /// Create new import options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set title override
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set author override
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set whether to extract cover art
    pub fn with_extract_cover(mut self, extract: bool) -> Self {
        self.extract_cover = extract;
        self
    }

    /// Set whether to overwrite existing books
    pub fn with_overwrite_existing(mut self, overwrite: bool) -> Self {
        self.overwrite_existing = overwrite;
        self
    }

    /// Set whether to skip files with errors
    pub fn with_skip_on_error(mut self, skip: bool) -> Self {
        self.skip_on_error = skip;
        self
    }
}

/// Book importer for adding audiobooks to the library
pub struct BookImporter {
    pool: DbPool,
    metadata_extractor: MetadataExtractor,
}

impl BookImporter {
    /// Create a new book importer
    pub fn new(pool: DbPool) -> Self {
        let metadata_extractor =
            MetadataExtractor::new().expect("Failed to initialize metadata extractor");

        Self {
            pool,
            metadata_extractor,
        }
    }

    /// Import a single audiobook file
    pub async fn import_file<P: AsRef<Path>>(
        &self,
        path: P,
        options: ImportOptions,
    ) -> Result<Book> {
        let path = path.as_ref();

        info!("Importing audiobook from: {}", path.display());

        // Validate file exists
        if !path.exists() {
            return Err(LibraryError::FileNotFound(path.display().to_string()));
        }

        // Validate file is actually a file (not a directory)
        if !path.is_file() {
            return Err(LibraryError::InvalidFile(format!(
                "Path is not a file: {}",
                path.display()
            )));
        }

        // Check if file is supported
        if !MetadataExtractor::is_supported(path) {
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown");
            return Err(LibraryError::UnsupportedFormat(format!(
                "Unsupported file format: .{}",
                extension
            )));
        }

        // Check if book already exists in database (by file path)
        let canonical_path = self.canonicalize_path(path)?;
        if let Some(existing_book) = self.find_by_path(&canonical_path).await? {
            if !options.overwrite_existing {
                return Err(LibraryError::ImportFailed(format!(
                    "Book already exists in library: {}",
                    existing_book.title
                )));
            }
            debug!("Overwriting existing book: {}", existing_book.title);
        }

        // Extract metadata
        let metadata = self.extract_metadata(path)?;

        // Apply any overrides from options
        let metadata = self.apply_options(metadata, &options);

        // Convert metadata to Book
        let mut book = self.metadata_extractor.to_book(path, metadata);

        // Use canonical path for storage
        book.file_path = canonical_path;

        // Insert into database
        books::create_book(&self.pool, &book)
            .await
            .map_err(LibraryError::Database)?;

        info!("Successfully imported: {}", book.title);

        Ok(book)
    }

    /// Import multiple audiobook files
    pub async fn import_files<P: AsRef<Path>>(
        &self,
        paths: &[P],
        options: ImportOptions,
    ) -> Result<Vec<Book>> {
        info!("Importing {} files", paths.len());

        let mut books = Vec::new();
        let mut errors = Vec::new();

        for (index, path) in paths.iter().enumerate() {
            let path = path.as_ref();
            debug!(
                "Processing file {}/{}: {}",
                index + 1,
                paths.len(),
                path.display()
            );

            match self.import_file(path, options.clone()).await {
                Ok(book) => {
                    books.push(book);
                }
                Err(e) => {
                    if options.skip_on_error {
                        warn!("Skipping file due to error: {} - {}", path.display(), e);
                        errors.push((path.to_path_buf(), e));
                    } else {
                        return Err(LibraryError::ImportFailed(format!(
                            "Failed to import {}: {}",
                            path.display(),
                            e
                        )));
                    }
                }
            }
        }

        if !errors.is_empty() {
            warn!(
                "Imported {}/{} files successfully ({} errors)",
                books.len(),
                paths.len(),
                errors.len()
            );
        } else {
            info!("Successfully imported all {} files", books.len());
        }

        Ok(books)
    }

    /// Import all audiobooks from a directory recursively
    pub async fn import_directory<P: AsRef<Path>>(
        &self,
        directory: P,
        options: ImportOptions,
    ) -> Result<Vec<Book>> {
        let directory = directory.as_ref();

        info!("Importing from directory: {}", directory.display());

        if !directory.exists() {
            return Err(LibraryError::FileNotFound(directory.display().to_string()));
        }

        if !directory.is_dir() {
            return Err(LibraryError::InvalidFile(format!(
                "Path is not a directory: {}",
                directory.display()
            )));
        }

        // Scan directory for audio files
        let audio_files = self.scan_directory(directory)?;

        info!("Found {} audio files in directory", audio_files.len());

        // Import all found files
        self.import_files(&audio_files, options).await
    }

    /// Scan directory recursively for audio files
    fn scan_directory(&self, directory: &Path) -> Result<Vec<PathBuf>> {
        let mut audio_files = Vec::new();

        let walker = walkdir::WalkDir::new(directory)
            .follow_links(false)
            .into_iter();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Error walking directory: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if MetadataExtractor::is_supported(path) {
                audio_files.push(path.to_path_buf());
            }
        }

        Ok(audio_files)
    }

    /// Extract metadata from an audio file
    fn extract_metadata(&self, path: &Path) -> Result<ExtractedMetadata> {
        debug!("Extracting metadata from: {}", path.display());

        self.metadata_extractor
            .extract(path)
            .map_err(|e| LibraryError::MetadataError(format!("{}", e)))
    }

    /// Apply import options to metadata
    fn apply_options(
        &self,
        mut metadata: ExtractedMetadata,
        options: &ImportOptions,
    ) -> ExtractedMetadata {
        if let Some(ref title) = options.title {
            metadata.title = Some(title.clone());
        }

        if let Some(ref author) = options.author {
            metadata.author = Some(author.clone());
        }

        if !options.extract_cover {
            metadata.cover_art = None;
        }

        metadata
    }

    /// Canonicalize a file path
    fn canonicalize_path(&self, path: &Path) -> Result<PathBuf> {
        path.canonicalize().map_err(|e| LibraryError::Io(e))
    }

    /// Find a book by its file path
    async fn find_by_path(&self, path: &Path) -> Result<Option<Book>> {
        let path_str = path.to_string_lossy().to_string();

        let all_books = books::list_books(&self.pool)
            .await
            .map_err(LibraryError::Database)?;

        for book in all_books {
            if book.file_path.to_string_lossy() == path_str {
                return Ok(Some(book));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use storystream_database::{
        connection::{connect, DatabaseConfig},
        migrations::run_migrations,
    };
    use tempfile::{NamedTempFile, TempDir};

    async fn setup_test_db() -> Result<(DbPool, NamedTempFile)> {
        let temp_file = NamedTempFile::new().map_err(LibraryError::Io)?;

        let db_path = temp_file
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path encoding".to_string()))?;

        let config = DatabaseConfig::new(db_path);
        let pool = connect(config).await.map_err(LibraryError::Database)?;

        run_migrations(&pool)
            .await
            .map_err(LibraryError::Database)?;

        Ok((pool, temp_file))
    }

    #[test]
    fn test_import_options_default() {
        let options = ImportOptions::default();
        assert!(options.title.is_none());
        assert!(options.author.is_none());
        assert!(options.extract_cover);
        assert!(!options.overwrite_existing);
        assert!(!options.skip_on_error);
    }

    #[test]
    fn test_import_options_builder() {
        let options = ImportOptions::new()
            .with_title("Custom Title")
            .with_author("Custom Author")
            .with_extract_cover(false)
            .with_overwrite_existing(true)
            .with_skip_on_error(true);

        assert_eq!(options.title, Some("Custom Title".to_string()));
        assert_eq!(options.author, Some("Custom Author".to_string()));
        assert!(!options.extract_cover);
        assert!(options.overwrite_existing);
        assert!(options.skip_on_error);
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
        assert!(matches!(result, Err(LibraryError::FileNotFound(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_import_unsupported_format() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;
        let importer = BookImporter::new(pool);

        // Create a temp file with unsupported extension
        let temp_file = NamedTempFile::with_suffix(".txt").map_err(LibraryError::Io)?;

        std::fs::write(temp_file.path(), b"not audio").map_err(LibraryError::Io)?;

        let result = importer
            .import_file(temp_file.path(), ImportOptions::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(LibraryError::UnsupportedFormat(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_import_directory_as_file() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;
        let importer = BookImporter::new(pool);

        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

        let result = importer
            .import_file(temp_dir.path(), ImportOptions::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(LibraryError::InvalidFile(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_import_files_with_skip_on_error() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;
        let importer = BookImporter::new(pool);

        // Create mix of valid and invalid files
        let invalid_file = NamedTempFile::with_suffix(".txt").map_err(LibraryError::Io)?;
        std::fs::write(invalid_file.path(), b"not audio").map_err(LibraryError::Io)?;

        let paths = vec![
            PathBuf::from("/nonexistent1.mp3"),
            invalid_file.path().to_path_buf(),
            PathBuf::from("/nonexistent2.mp3"),
        ];

        let options = ImportOptions::new().with_skip_on_error(true);
        let result = importer.import_files(&paths, options).await;

        // Should succeed but with empty results since all failed
        assert!(result.is_ok());
        let books = result.unwrap();
        assert_eq!(books.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_import_files_without_skip_fails_on_first_error() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;
        let importer = BookImporter::new(pool);

        let paths = vec![
            PathBuf::from("/nonexistent1.mp3"),
            PathBuf::from("/nonexistent2.mp3"),
        ];

        let options = ImportOptions::new().with_skip_on_error(false);
        let result = importer.import_files(&paths, options).await;

        // Should fail on first error
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_scan_empty_directory() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;
        let importer = BookImporter::new(pool);

        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

        let files = importer.scan_directory(temp_dir.path())?;
        assert_eq!(files.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_import_directory_nonexistent() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;
        let importer = BookImporter::new(pool);

        let result = importer
            .import_directory("/nonexistent/directory", ImportOptions::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(LibraryError::FileNotFound(_))));

        Ok(())
    }

    #[tokio::test]
    async fn test_import_directory_is_file() -> Result<()> {
        let (pool, _temp) = setup_test_db().await?;
        let importer = BookImporter::new(pool);

        let temp_file = NamedTempFile::new().map_err(LibraryError::Io)?;

        let result = importer
            .import_directory(temp_file.path(), ImportOptions::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(LibraryError::InvalidFile(_))));

        Ok(())
    }
}
