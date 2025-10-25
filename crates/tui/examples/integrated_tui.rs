// crates/tui/examples/integrated_tui.rs
//! Fully Integrated TUI Example
//!
//! This example demonstrates the complete StoryStream TUI with:
//! - Real audio playback via MediaEngine
//! - Actual library management
//! - Database persistence
//! - Configuration system
//!
//! Usage:
//!   cargo run --example integrated_tui

use storystream_tui::IntegratedTuiApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for debugging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("═══════════════════════════════════════");
    println!("  StoryStream - Integrated TUI Demo");
    println!("═══════════════════════════════════════\n");
    println!("Features:");
    println!("  ✓ Real audio playback");
    println!("  ✓ Library management");
    println!("  ✓ Database integration");
    println!("  ✓ Full keyboard & mouse support\n");
    println!("Controls:");
    println!("  Tab      - Switch views");
    println!("  Space    - Play/Pause");
    println!("  Enter    - Select item");
    println!("  ↑/↓      - Navigate");
    println!("  ←/→      - Seek ±10s");
    println!("  +/-      - Volume");
    println!("  [/]      - Speed");
    println!("  t        - Toggle theme");
    println!("  h        - Help");
    println!("  q        - Quit\n");
    println!("Starting in 3 seconds...\n");

    // Give user time to read instructions
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Create and run integrated TUI
    let mut app = IntegratedTuiApp::new().await?;
    app.run().await?;

    println!("\n═══════════════════════════════════════");
    println!("  Thanks for using StoryStream!");
    println!("═══════════════════════════════════════\n");

    Ok(())
}