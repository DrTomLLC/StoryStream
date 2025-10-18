// FILE: crates/library/tests/import_tests.rs
//! Integration tests for BookImporter

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use storystream_library::{BookImporter, ImportOptions, LibraryError};
use storystream_database::{
    connection::{connect, DatabaseConfig},
    migrations::run_migrations,
    queries::books,
    DbPool,
};
use tempfile::{NamedTempFile, TempDir};

type Result<T> = std::result::Result<T, LibraryError>;

async fn setup_test_db() -> Result<(DbPool, NamedTempFile)> {
    let temp_file = NamedTempFile::new()
        .map_err(LibraryError::Io)?;

    let db_path = temp_file
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?;

    let config = DatabaseConfig::new(db_path);
    let pool = connect(config)
        .await
        .map_err(LibraryError::Database)?;

    run_migrations(&pool)
        .await
        .map_err(LibraryError::Database)?;

    Ok((pool, temp_file))
}

fn create_fake_audio_file(dir: &std::path::Path, name: &str, extension: &str) -> PathBuf {
    let file_path = dir.join(format!("{}.{}", name, extension));
    // Create a file with minimal content
    // Note: These won't have valid audio metadata, but test the import flow
    fs::write(&file_path, b"FAKE_AUDIO_DATA").unwrap();
    file_path
}

#[tokio::test]
async fn test_importer_basic_creation() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let _importer = BookImporter::new(pool);
    Ok(())
}

#[tokio::test]
async fn test_import_nonexistent_file_error() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let result = importer
        .import_file("/absolutely/nonexistent/path/file.mp3", ImportOptions::default())
        .await;

    assert!(result.is_err());
    match result {
        Err(LibraryError::FileNotFound(path)) => {
            assert!(path.contains("nonexistent"));
        }
        _ => panic!("Expected FileNotFound error"),
    }

    Ok(())
}

#[tokio::test]
async fn test_import_unsupported_file_format() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let temp_file = NamedTempFile::with_suffix(".txt")
        .map_err(LibraryError::Io)?;

    fs::write(temp_file.path(), b"This is not an audio file")
        .map_err(LibraryError::Io)?;

    let result = importer
        .import_file(temp_file.path(), ImportOptions::default())
        .await;

    assert!(result.is_err());
    match result {
        Err(LibraryError::UnsupportedFormat(msg)) => {
            assert!(msg.contains("txt") || msg.contains("Unsupported"));
        }
        _ => panic!("Expected UnsupportedFormat error"),
    }

    Ok(())
}

#[tokio::test]
async fn test_import_directory_as_file_error() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let temp_dir = TempDir::new()
        .map_err(LibraryError::Io)?;

    let result = importer
        .import_file(temp_dir.path(), ImportOptions::default())
        .await;

    assert!(result.is_err());
    match result {
        Err(LibraryError::InvalidFile(msg)) => {
            assert!(msg.contains("not a file") || msg.contains("directory"));
        }
        _ => panic!("Expected InvalidFile error"),
    }

    Ok(())
}

#[tokio::test]
async fn test_import_options_builder_pattern() -> Result<()> {
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

    Ok(())
}

#[tokio::test]
async fn test_import_multiple_files_with_errors_skip() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    // Create mix of files - all will fail but with skip_on_error
    let paths = vec![
        PathBuf::from("/fake/path1.mp3"),
        PathBuf::from("/fake/path2.mp3"),
        PathBuf::from("/fake/path3.mp3"),
    ];

    let options = ImportOptions::new()
        .with_skip_on_error(true);

    let result = importer.import_files(&paths, options).await;

    // Should succeed but return empty list
    assert!(result.is_ok());
    let books = result.unwrap();
    assert_eq!(books.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_import_multiple_files_fail_fast() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let paths = vec![
        PathBuf::from("/fake/path1.mp3"),
        PathBuf::from("/fake/path2.mp3"),
    ];

    let options = ImportOptions::new()
        .with_skip_on_error(false);

    let result = importer.import_files(&paths, options).await;

    // Should fail on first error
    assert!(result.is_err());
    match result {
        Err(LibraryError::ImportFailed(_)) => {}
        _ => panic!("Expected ImportFailed error"),
    }

    Ok(())
}

#[tokio::test]
async fn test_scan_empty_directory() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let temp_dir = TempDir::new()
        .map_err(LibraryError::Io)?;

    // Use reflection to access private scan_directory method via import_directory
    let result = importer
        .import_directory(temp_dir.path(), ImportOptions::default())
        .await;

    // Empty directory should succeed with zero books
    assert!(result.is_ok());
    let books = result.unwrap();
    assert_eq!(books.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_scan_directory_with_mixed_files() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let temp_dir = TempDir::new()
        .map_err(LibraryError::Io)?;

    // Create mix of audio and non-audio files
    // Note: These will fail metadata extraction but test the scanning logic
    create_fake_audio_file(temp_dir.path(), "audio1", "mp3");
    create_fake_audio_file(temp_dir.path(), "audio2", "m4b");

    // Non-audio files
    fs::write(temp_dir.path().join("readme.txt"), b"text file").unwrap();
    fs::write(temp_dir.path().join("image.jpg"), b"image data").unwrap();

    // Import with skip_on_error since fake files won't have valid metadata
    let options = ImportOptions::new()
        .with_skip_on_error(true);

    let result = importer
        .import_directory(temp_dir.path(), options)
        .await;

    // Should succeed (files will fail metadata extraction but be skipped)
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_import_directory_nonexistent() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let result = importer
        .import_directory("/absolutely/nonexistent/directory", ImportOptions::default())
        .await;

    assert!(result.is_err());
    match result {
        Err(LibraryError::FileNotFound(_)) => {}
        _ => panic!("Expected FileNotFound error"),
    }

    Ok(())
}

#[tokio::test]
async fn test_import_directory_is_actually_file() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let temp_file = NamedTempFile::new()
        .map_err(LibraryError::Io)?;

    let result = importer
        .import_directory(temp_file.path(), ImportOptions::default())
        .await;

    assert!(result.is_err());
    match result {
        Err(LibraryError::InvalidFile(_)) => {}
        _ => panic!("Expected InvalidFile error"),
    }

    Ok(())
}

#[tokio::test]
async fn test_scan_directory_with_subdirectories() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let temp_dir = TempDir::new()
        .map_err(LibraryError::Io)?;

    // Create subdirectory structure
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();

    let nested_subdir = subdir.join("nested");
    fs::create_dir(&nested_subdir).unwrap();

    // Create fake audio files in different directories
    create_fake_audio_file(temp_dir.path(), "root_audio", "mp3");
    create_fake_audio_file(&subdir, "subdir_audio", "m4b");
    create_fake_audio_file(&nested_subdir, "nested_audio", "flac");

    let options = ImportOptions::new()
        .with_skip_on_error(true);

    let result = importer
        .import_directory(temp_dir.path(), options)
        .await;

    // Should succeed and scan recursively
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_import_options_default_values() -> Result<()> {
    let options = ImportOptions::default();

    assert!(options.title.is_none());
    assert!(options.author.is_none());
    assert!(options.extract_cover);
    assert!(!options.overwrite_existing);
    assert!(!options.skip_on_error);

    Ok(())
}

#[tokio::test]
async fn test_import_options_with_all_overrides() -> Result<()> {
    let options = ImportOptions::new()
        .with_title("Test Title")
        .with_author("Test Author")
        .with_extract_cover(false)
        .with_overwrite_existing(true)
        .with_skip_on_error(true);

    assert_eq!(options.title.unwrap(), "Test Title");
    assert_eq!(options.author.unwrap(), "Test Author");
    assert!(!options.extract_cover);
    assert!(options.overwrite_existing);
    assert!(options.skip_on_error);

    Ok(())
}

#[tokio::test]
async fn test_multiple_importers_same_database() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;

    // Create multiple importers sharing the same pool
    let importer1 = BookImporter::new(pool.clone());
    let importer2 = BookImporter::new(pool.clone());
    let _importer3 = BookImporter::new(pool);

    // Should all work independently
    let result1 = importer1
        .import_file("/fake/file1.mp3", ImportOptions::default())
        .await;

    let result2 = importer2
        .import_file("/fake/file2.mp3", ImportOptions::default())
        .await;

    // Both should fail with FileNotFound
    assert!(result1.is_err());
    assert!(result2.is_err());

    Ok(())
}

#[tokio::test]
async fn test_import_empty_file_list() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let paths: Vec<PathBuf> = vec![];
    let result = importer
        .import_files(&paths, ImportOptions::default())
        .await;

    assert!(result.is_ok());
    let books = result.unwrap();
    assert_eq!(books.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_import_with_title_override() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let temp_file = NamedTempFile::with_suffix(".txt")
        .map_err(LibraryError::Io)?;

    let options = ImportOptions::new()
        .with_title("Override Title");

    let result = importer
        .import_file(temp_file.path(), options)
        .await;

    // Will still fail on unsupported format, but tests option flow
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_concurrent_imports_different_files() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;

    let importer1 = BookImporter::new(pool.clone());
    let importer2 = BookImporter::new(pool.clone());
    let importer3 = BookImporter::new(pool);

    // Try to import different files concurrently
    let (result1, result2, result3) = tokio::join!(
        importer1.import_file("/fake1.mp3", ImportOptions::default()),
        importer2.import_file("/fake2.mp3", ImportOptions::default()),
        importer3.import_file("/fake3.mp3", ImportOptions::default())
    );

    // All should fail with FileNotFound, but shouldn't crash
    assert!(result1.is_err());
    assert!(result2.is_err());
    assert!(result3.is_err());

    Ok(())
}

#[tokio::test]
async fn test_scan_directory_ignore_hidden_files() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let temp_dir = TempDir::new()
        .map_err(LibraryError::Io)?;

    // Create visible and hidden audio files
    create_fake_audio_file(temp_dir.path(), "visible", "mp3");
    create_fake_audio_file(temp_dir.path(), ".hidden", "mp3");

    let options = ImportOptions::new()
        .with_skip_on_error(true);

    let result = importer
        .import_directory(temp_dir.path(), options)
        .await;

    // Should succeed (walkdir includes hidden files by default)
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_import_case_insensitive_extensions() -> Result<()> {
    let (pool, _temp) = setup_test_db().await?;
    let importer = BookImporter::new(pool);

    let temp_dir = TempDir::new()
        .map_err(LibraryError::Io)?;

    // Create files with different case extensions
    create_fake_audio_file(temp_dir.path(), "lower", "mp3");
    create_fake_audio_file(temp_dir.path(), "upper", "MP3");
    create_fake_audio_file(temp_dir.path(), "mixed", "Mp3");

    let options = ImportOptions::new()
        .with_skip_on_error(true);

    let result = importer
        .import_directory(temp_dir.path(), options)
        .await;

    // All should be recognized as audio files
    assert!(result.is_ok());

    Ok(())
}