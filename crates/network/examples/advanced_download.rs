// crates/network/examples/advanced_download.rs
//! Advanced download example demonstrating all features

use std::path::PathBuf;
use std::sync::Arc;
use storystream_network::{
    AdvancedDownloadManager, Client, DownloadManagerConfig, DownloadTask, Priority,
};
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🚀 StoryStream Advanced Download Manager Demo\n");

    // Create HTTP client
    let client = Client::new()?;

    // Configure download manager
    let config = DownloadManagerConfig {
        max_concurrent: 3,
        auto_resume: true,
        max_resume_attempts: 5,
        bandwidth_limit: Some(2_000_000), // 2 MB/s
        chunk_size: 8192,
        verify_integrity: false,
    };

    println!("📋 Configuration:");
    println!("  Max concurrent: {}", config.max_concurrent);
    println!("  Auto-resume: {}", config.auto_resume);
    println!("  Bandwidth limit: 2 MB/s");
    println!();

    let manager = Arc::new(AdvancedDownloadManager::new(client, config));

    // Define downloads
    let downloads = vec![
        (
            "rust-logo",
            "https://www.rust-lang.org/static/images/rust-logo-blk.svg",
            Priority::High,
        ),
        (
            "robots-txt",
            "https://www.rust-lang.org/robots.txt",
            Priority::Normal,
        ),
        (
            "manifest",
            "https://www.rust-lang.org/manifest.json",
            Priority::Low,
        ),
    ];

    println!("📥 Queuing {} downloads...\n", downloads.len());

    let temp_dir = tempfile::tempdir()?;

    // Enqueue downloads
    for (id, url, priority) in &downloads {
        let destination = temp_dir.path().join(format!("{}.download", id));

        let id_clone = id.to_string();
        let progress_callback = Arc::new(move |downloaded: u64, total: Option<u64>| {
            if let Some(total) = total {
                let pct = (downloaded as f64 / total as f64) * 100.0;
                print!(
                    "\r  {} - {:.1}% ({}/{} bytes)",
                    id_clone, pct, downloaded, total
                );
            } else {
                print!("\r  {} - {} bytes", id_clone, downloaded);
            }
            std::io::Write::flush(&mut std::io::stdout()).ok();
        });

        let task = DownloadTask::new(id.to_string(), url.to_string(), destination)
            .with_priority(*priority)
            .with_progress_callback(progress_callback);

        manager.enqueue(task).await?;
        println!("  ✓ Queued: {} (Priority: {:?})", id, priority);
    }

    println!("\n⏳ Starting download manager...\n");

    // Start manager
    let manager_clone = Arc::clone(&manager);
    let manager_handle = tokio::spawn(async move {
        manager_clone.start().await;
    });

    // Monitor progress
    let mut last_active = 0;
    let mut last_queued = downloads.len();

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let active = manager.active_count().await;
        let queued = manager.queue_length().await;

        if active != last_active || queued != last_queued {
            println!("\n📊 Status: {} active, {} queued", active, queued);
            last_active = active;
            last_queued = queued;
        }

        if active == 0 && queued == 0 {
            break;
        }
    }

    println!("\n\n✅ All downloads completed!\n");

    manager.shutdown().await?;
    manager_handle.abort();

    // Show results
    println!("📁 Downloaded files:");
    for entry in std::fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        println!(
            "  {} ({} bytes)",
            entry.file_name().to_string_lossy(),
            metadata.len()
        );
    }

    println!("\n🎉 Demo completed successfully!");

    Ok(())
}
