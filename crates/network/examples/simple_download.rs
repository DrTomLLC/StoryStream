// crates/network/examples/simple_download.rs
//! Simple download example showing basic usage

use storystream_network::{Client, DownloadManager, ProgressTracker};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("📥 Simple Download Example\n");

    // Create HTTP client
    let client = Client::new()?;
    let manager = DownloadManager::new(client);

    // Create temp directory
    let temp_dir = tempfile::tempdir()?;
    let destination = temp_dir.path().join("robots.txt");

    // Create progress tracker
    let progress = ProgressTracker::new(None);
    let progress_clone = progress.clone();

    // Spawn progress monitoring
    let monitor = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if let Some(p) = progress_clone.get() {
                if let Some(pct) = p.percentage() {
                    print!("\r Progress: {:.1}%", pct);
                    std::io::Write::flush(&mut std::io::stdout()).ok();
                }
            }
        }
    });

    // Download file
    println!("Downloading https://www.rust-lang.org/robots.txt...\n");

    match manager
        .download_file(
            "https://www.rust-lang.org/robots.txt",
            &destination,
            Some(progress),
        )
        .await
    {
        Ok(bytes) => {
            monitor.abort();
            println!("\n\n✅ Download complete: {} bytes", bytes);

            // Show content
            let content = std::fs::read_to_string(&destination)?;
            println!("\n📄 File content:\n");
            for (i, line) in content.lines().take(10).enumerate() {
                println!("  {}: {}", i + 1, line);
            }

            if content.lines().count() > 10 {
                println!("  ... ({} more lines)", content.lines().count() - 10);
            }
        }
        Err(e) => {
            monitor.abort();
            eprintln!("\n❌ Download failed: {}", e);
        }
    }

    Ok(())
}