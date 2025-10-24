// crates/tui/examples/fully_functional_demo.rs
//! Fully functional TUI demo with mouse support and interactive features

use storystream_tui::{TuiApp, TuiResult};

fn main() -> TuiResult<()> {
    // Initialize and run the fully functional TUI
    let mut app = TuiApp::new()?;

    println!("Starting StoryStream TUI Demo...");
    println!("Features:");
    println!("  - Full mouse support (click, scroll)");
    println!("  - All 8 views accessible");
    println!("  - Theme switching (press 't')");
    println!("  - Complete keyboard controls");
    println!("Press 'h' for help, 'q' to quit");

    std::thread::sleep(std::time::Duration::from_secs(10));

    app.run()?;

    println!("Thanks for using StoryStream!");
    Ok(())
}
