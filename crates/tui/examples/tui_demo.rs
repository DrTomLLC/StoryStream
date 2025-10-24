// crates/tui/examples/tui_demo.rs
// Demo of the StoryStream TUI

use storystream_tui::TuiApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and run the TUI
    let mut app = TuiApp::new()?;
    app.run()?;

    Ok(())
}
