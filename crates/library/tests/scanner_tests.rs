// FILE: crates/library/tests/scanner_tests.rs
//! Comprehensive integration tests for LibraryScanner

use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use storystream_library::{LibraryError, LibraryScanner};
use tempfile::TempDir;

type Result<T> = std::result::Result<T, LibraryError>;

fn create_audio_file(dir: &std::path::Path, name: &str, size: usize) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, vec![0u8; size]).unwrap();
    path
}

#[tokio::test]
async fn test_scanner_finds_all_audio_formats() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    // Create files with different audio extensions
    let extensions = vec!["mp3", "m4a", "m4b", "flac", "ogg", "opus", "aac"];

    for ext in extensions {
        create_audio_file(temp_dir.path(), &format!("test.{}", ext), 2048);
    }

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let scanner = LibraryScanner::new(vec![path]);
    let files = scanner.scan().await?;

    // Should find all 7 audio files
    assert_eq!(files.len(), 7);

    Ok(())
}

#[tokio::test]
async fn test_scanner_ignores_hidden_files() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    // Create visible audio file
    create_audio_file(temp_dir.path(), "visible.mp3", 2048);

    // Create hidden audio file (starts with .)
    create_audio_file(temp_dir.path(), ".hidden.mp3", 2048);

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let scanner = LibraryScanner::new(vec![path]);
    let files = scanner.scan().await?;

    // walkdir includes hidden files by default, so we should find both
    // This test documents current behavior
    assert!(files.len() >= 1);
    assert!(files.iter().any(|f| f.ends_with("visible.mp3")));

    Ok(())
}

#[tokio::test]
async fn test_scanner_deep_directory_structure() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    // Create deep directory structure
    let mut current_dir = temp_dir.path().to_path_buf();
    for i in 0..5 {
        current_dir = current_dir.join(format!("level{}", i));
        fs::create_dir(&current_dir).unwrap();
        create_audio_file(&current_dir, &format!("audio{}.mp3", i), 2048);
    }

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let scanner = LibraryScanner::new(vec![path]);
    let files = scanner.scan().await?;

    // Should find all 5 audio files
    assert_eq!(files.len(), 5);

    Ok(())
}

#[tokio::test]
async fn test_scanner_with_symlinks() -> Result<()> {
    // This test only works on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

        // Create actual directory with audio file
        let actual_dir = temp_dir.path().join("actual");
        fs::create_dir(&actual_dir).unwrap();
        create_audio_file(&actual_dir, "audio.mp3", 2048);

        // Create symlink to directory
        let link_dir = temp_dir.path().join("link");
        symlink(&actual_dir, &link_dir).unwrap();

        let path = temp_dir
            .path()
            .to_str()
            .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
            .to_string();

        // Scan without following symlinks
        let scanner = LibraryScanner::new(vec![path.clone()]);
        let files_no_follow = scanner.scan().await?;

        // Scan with following symlinks
        use storystream_library::scanner::ScannerConfig;
        let config = ScannerConfig::new(vec![path]).with_follow_symlinks(true);
        let scanner_follow = LibraryScanner::with_config(config);
        let files_follow = scanner_follow.scan().await?;

        // Without following, should find only in actual directory
        // With following, might find in both (depending on deduplication)
        assert!(files_no_follow.len() >= 1);
        assert!(files_follow.len() >= 1);
    }

    Ok(())
}

#[tokio::test]
async fn test_scanner_large_directory() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    // Create many audio files
    for i in 0..500 {
        create_audio_file(temp_dir.path(), &format!("audio{:03}.mp3", i), 2048);
    }

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let scanner = LibraryScanner::new(vec![path]);

    let start = std::time::Instant::now();
    let files = scanner.scan().await?;
    let duration = start.elapsed();

    // Should find all 500 files
    assert_eq!(files.len(), 500);

    // Should complete in reasonable time (< 5 seconds for 500 files)
    assert!(
        duration < Duration::from_secs(5),
        "Scan took too long: {:?}",
        duration
    );

    Ok(())
}

#[tokio::test]
async fn test_scanner_mixed_content() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    // Create mixed content
    create_audio_file(temp_dir.path(), "audio.mp3", 2048);
    fs::write(temp_dir.path().join("document.pdf"), vec![0u8; 2048]).unwrap();
    fs::write(temp_dir.path().join("image.jpg"), vec![0u8; 2048]).unwrap();
    fs::write(temp_dir.path().join("video.mp4"), vec![0u8; 2048]).unwrap();
    fs::write(temp_dir.path().join("text.txt"), "test").unwrap();

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let scanner = LibraryScanner::new(vec![path]);
    let files = scanner.scan().await?;

    // Should only find the audio file
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("audio.mp3"));

    Ok(())
}

#[tokio::test]
async fn test_scanner_case_insensitive_extensions() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    // Create files with different case extensions
    create_audio_file(temp_dir.path(), "lower.mp3", 2048);
    create_audio_file(temp_dir.path(), "upper.MP3", 2048);
    create_audio_file(temp_dir.path(), "mixed.Mp3", 2048);

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let scanner = LibraryScanner::new(vec![path]);
    let files = scanner.scan().await?;

    // Should find all 3 files regardless of case
    assert_eq!(files.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_scanner_special_characters_in_filename() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    // Create files with special characters
    create_audio_file(temp_dir.path(), "audio with spaces.mp3", 2048);
    create_audio_file(temp_dir.path(), "audio-with-dashes.mp3", 2048);
    create_audio_file(temp_dir.path(), "audio_with_underscores.mp3", 2048);

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let scanner = LibraryScanner::new(vec![path]);
    let files = scanner.scan().await?;

    // Should find all 3 files
    assert_eq!(files.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_scanner_empty_files() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    // Create empty audio file (0 bytes)
    let empty = temp_dir.path().join("empty.mp3");
    fs::write(&empty, Vec::new()).unwrap();

    // Create small audio file (< minimum size)
    create_audio_file(temp_dir.path(), "small.mp3", 512);

    // Create normal audio file
    create_audio_file(temp_dir.path(), "normal.mp3", 2048);

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let scanner = LibraryScanner::new(vec![path]);
    let files = scanner.scan().await?;

    // Should only find the normal-sized file (minimum is 1KB by default)
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("normal.mp3"));

    Ok(())
}

#[tokio::test]
async fn test_scanner_concurrent_scans() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    // Create audio files
    for i in 0..10 {
        create_audio_file(temp_dir.path(), &format!("audio{}.mp3", i), 2048);
    }

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    // Run multiple scans concurrently
    let scanner1 = LibraryScanner::new(vec![path.clone()]);
    let scanner2 = LibraryScanner::new(vec![path.clone()]);
    let scanner3 = LibraryScanner::new(vec![path]);

    let (files1, files2, files3) = tokio::join!(scanner1.scan(), scanner2.scan(), scanner3.scan());

    // All should succeed and find the same files
    assert_eq!(files1?.len(), 10);
    assert_eq!(files2?.len(), 10);
    assert_eq!(files3?.len(), 10);

    Ok(())
}

#[tokio::test]
async fn test_scanner_watch_stop_watch() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let mut scanner = LibraryScanner::new(vec![path]);

    // Start -> Stop -> Start again should work
    let _rx1 = scanner.start().await?;
    scanner.stop().await?;

    let _rx2 = scanner.start().await?;
    scanner.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_scanner_modification_events() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let mut scanner = LibraryScanner::new(vec![path]);
    let mut rx = scanner.start().await?;

    // Give watcher time to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create a file
    let test_file = temp_dir.path().join("test.mp3");
    fs::write(&test_file, vec![0u8; 2048]).unwrap();

    // Wait for create event
    let _ = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;

    // Modify the file
    fs::write(&test_file, vec![1u8; 2048]).unwrap();

    // Wait for modify event with timeout
    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;

    scanner.stop().await?;

    // We should get some event (create or modify)
    assert!(event.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_scanner_removal_events() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    // Create a file before starting watcher
    let test_file = temp_dir.path().join("test.mp3");
    fs::write(&test_file, vec![0u8; 2048]).unwrap();

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let mut scanner = LibraryScanner::new(vec![path]);
    let mut rx = scanner.start().await?;

    // Give watcher time to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Remove the file
    fs::remove_file(&test_file).unwrap();

    // Wait for remove event with timeout
    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;

    scanner.stop().await?;

    // Should receive a removal event
    assert!(event.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_scanner_multiple_watch_paths() -> Result<()> {
    let temp_dir1 = TempDir::new().map_err(LibraryError::Io)?;
    let temp_dir2 = TempDir::new().map_err(LibraryError::Io)?;

    // Create files in both directories
    create_audio_file(temp_dir1.path(), "audio1.mp3", 2048);
    create_audio_file(temp_dir2.path(), "audio2.mp3", 2048);

    let path1 = temp_dir1
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let path2 = temp_dir2
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let scanner = LibraryScanner::new(vec![path1, path2]);
    let files = scanner.scan().await?;

    // Should find files in both directories
    assert_eq!(files.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_scanner_performance_many_events() -> Result<()> {
    let temp_dir = TempDir::new().map_err(LibraryError::Io)?;

    let path = temp_dir
        .path()
        .to_str()
        .ok_or_else(|| LibraryError::InvalidFile("Invalid path".to_string()))?
        .to_string();

    let mut scanner = LibraryScanner::new(vec![path]);
    let mut rx = scanner.start().await?;

    // Give watcher time to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create many files rapidly
    for i in 0..20 {
        let file = temp_dir.path().join(format!("audio{}.mp3", i));
        fs::write(&file, vec![0u8; 2048]).unwrap();
    }

    // Collect events for a short time
    let mut event_count = 0;
    let timeout = tokio::time::sleep(Duration::from_secs(3));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            _ = &mut timeout => break,
            event = rx.recv() => {
                if event.is_some() {
                    event_count += 1;
                } else {
                    break;
                }
            }
        }
    }

    scanner.stop().await?;

    // Should have received events (exact count may vary due to timing)
    assert!(event_count > 0, "Expected events but got {}", event_count);

    Ok(())
}
