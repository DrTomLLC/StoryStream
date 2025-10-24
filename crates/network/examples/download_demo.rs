// crates/network/examples/download_demo.rs
//! Demonstration of network download capabilities

use storystream_network::{Client, ConnectivityChecker, DownloadManager, ProgressTracker};

#[tokio::main]
async fn main() {
    println!("StoryStream Network Module Demo");
    println!("================================\n");

    // Create client
    let client = match Client::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create client: {}", e);
            return;
        }
    };

    // Check connectivity
    println!("1. Checking network connectivity...");
    let checker = ConnectivityChecker::new(client.clone());

    match checker.check().await {
        Ok(()) => println!("   ✓ Network is available\n"),
        Err(e) => {
            println!("   ✗ Network check failed: {}\n", e);
            return;
        }
    }

    // Test HEAD request
    println!("2. Testing HEAD request...");
    let test_url = "https://www.rust-lang.org";

    match client.content_length(test_url).await {
        Ok(Some(size)) => println!("   ✓ Content length: {} bytes\n", size),
        Ok(None) => println!("   ✓ Content length: Unknown\n"),
        Err(e) => println!("   ✗ HEAD request failed: {}\n", e),
    }

    // Test download to memory
    println!("3. Downloading small file to memory...");
    let manager = DownloadManager::new(client);

    match manager
        .download_string("https://www.rust-lang.org/robots.txt")
        .await
    {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().take(3).collect();
            println!("   ✓ Downloaded {} bytes", content.len());
            println!("   First 3 lines:");
            for line in lines {
                println!("     {}", line);
            }
            println!();
        }
        Err(e) => println!("   ✗ Download failed: {}\n", e),
    }

    // Simulate download with progress
    println!("4. Simulating download with progress tracking...");
    let progress = ProgressTracker::new(Some(10000));

    for _i in 1..=10 {
        progress.update(1000);

        if let Some(pct) = progress.percentage() {
            let bars = "█".repeat((pct / 10.0) as usize);
            let spaces = " ".repeat(10 - (pct / 10.0) as usize);
            print!("\r   [{bars}{spaces}] {:.1}%", pct);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!("\n   ✓ Download complete\n");

    println!("Demo completed successfully!");
}
