// crates/network/examples/resume_demo.rs
//! Resume capability demonstration
//!
//! This example shows:
//! - Saving resume information
//! - Resuming interrupted downloads
//! - Cleanup of old resume data

use std::time::Duration;
use storystream_network::{can_resume, ResumeInfo, ResumeManager};
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🔄 Download Resume Capability Demo\n");

    // Create temp directories
    let temp_dir = tempfile::tempdir()?;
    let metadata_dir = tempfile::tempdir()?;

    // Create resume manager
    let manager = ResumeManager::new(metadata_dir.path())?;

    println!("1️⃣ Simulating interrupted download...\n");

    // Simulate a partial download
    let file_path = temp_dir.path().join("large_file.dat");
    let mut file = fs::File::create(&file_path).await?;

    // Write 5MB of data
    let partial_data = vec![0u8; 5_000_000];
    file.write_all(&partial_data).await?;
    file.flush().await?;
    drop(file);

    println!("  ✓ Wrote 5 MB to disk");

    // Create resume information
    let resume_info = ResumeInfo::new(5_000_000)
        .with_etag("abc123xyz".to_string())
        .with_total_size(10_000_000); // 10MB total

    // Save resume info
    manager.save("large_download", &resume_info).await?;

    println!("  ✓ Saved resume information");
    println!("    - Downloaded: {} bytes", resume_info.bytes_downloaded);
    println!(
        "    - Total size: {} bytes",
        resume_info.total_size.unwrap()
    );
    println!(
        "    - Progress: {:.1}%",
        resume_info.progress_percentage().unwrap()
    );
    println!("    - ETag: {}", resume_info.etag.as_ref().unwrap());

    println!("\n2️⃣ Checking if download can be resumed...\n");

    // Check if we can resume
    let can_resume_download = can_resume(&file_path, &resume_info).await?;
    println!("  ✓ Can resume: {}", can_resume_download);

    // Modify file size to simulate corruption
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&file_path)
        .await?;
    file.write_all(&vec![0u8; 3_000_000]).await?;
    drop(file);

    let can_resume_corrupted = can_resume(&file_path, &resume_info).await?;
    println!("  ✓ Can resume after corruption: {}", can_resume_corrupted);

    println!("\n3️⃣ Loading resume information...\n");

    // Load resume info
    if let Some(loaded) = manager.load("large_download").await? {
        println!("  ✓ Loaded resume data:");
        println!("    - Bytes downloaded: {}", loaded.bytes_downloaded);
        println!(
            "    - Progress: {:.1}%",
            loaded.progress_percentage().unwrap()
        );
        println!("    - Interrupted at: {}", loaded.interrupted_at);
        println!("    - Is complete: {}", loaded.is_complete());
    }

    println!("\n4️⃣ Creating multiple downloads...\n");

    // Create several resume infos
    for i in 1..=5 {
        let info = ResumeInfo::new(i * 1_000_000).with_total_size(10_000_000);
        manager.save(&format!("download_{}", i), &info).await?;
        println!(
            "  ✓ Saved download_{} ({:.1}%)",
            i,
            info.progress_percentage().unwrap()
        );
    }

    println!("\n5️⃣ Listing incomplete downloads...\n");

    let incomplete = manager.list_incomplete().await?;
    println!("  Found {} incomplete downloads:", incomplete.len());
    for (id, info) in &incomplete {
        println!(
            "    - {}: {:.1}% ({} / {} bytes)",
            id,
            info.progress_percentage().unwrap(),
            info.bytes_downloaded,
            info.total_size.unwrap()
        );
    }

    println!("\n6️⃣ Simulating old downloads...\n");

    // Create an old resume info
    let mut old_info = ResumeInfo::new(1_000_000);
    old_info.interrupted_at = chrono::Utc::now() - chrono::Duration::days(10);
    manager.save("old_download", &old_info).await?;
    println!("  ✓ Created download from 10 days ago");

    // Cleanup old downloads
    println!("\n7️⃣ Cleaning up old downloads (>7 days)...\n");
    let removed = manager.cleanup_old(7).await?;
    println!("  ✓ Removed {} old downloads", removed);

    // Verify old download is gone
    let has_old = manager.has_resume_info("old_download");
    println!("  ✓ Old download still exists: {}", has_old);

    println!("\n8️⃣ Deleting specific download...\n");

    manager.delete("download_1").await?;
    println!("  ✓ Deleted download_1");

    let remaining = manager.list_incomplete().await?;
    println!("  ✓ Remaining downloads: {}", remaining.len());

    println!("\n✅ Resume capability demo completed!\n");

    // Summary
    println!("📊 Summary:");
    println!("  - Resume information can be saved and loaded");
    println!("  - File integrity can be verified before resume");
    println!("  - Old resume data can be automatically cleaned up");
    println!("  - Progress tracking shows completion percentage");
    println!("  - ETag validation prevents resuming modified files");

    Ok(())
}
