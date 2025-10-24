// crates/cli/src/main.rs
//! StoryStream CLI - Command-line interface for the audiobook player

mod commands;
mod player;
mod tui_mode;

use anyhow::Result;
use clap::Parser;
use commands::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Execute the requested command
    match cli.command {
        Commands::Tui => {
            // Launch integrated TUI mode with real audio playback
            tui_mode::run_tui().await?;
        }
        Commands::Play {
            book,
            speed,
            volume,
        } => {
            println!("Playing: {}", book);
            if let Some(s) = speed {
                println!("  Speed: {}x", s);
            }
            if let Some(v) = volume {
                println!("  Volume: {}%", v);
            }
            println!("\nNote: Use 'storystream tui' for full interactive experience");
        }
        Commands::Pause => {
            println!("Pausing playback");
            println!("\nNote: Use 'storystream tui' for full interactive experience");
        }
        Commands::Resume => {
            println!("Resuming playback");
            println!("\nNote: Use 'storystream tui' for full interactive experience");
        }
        Commands::Stop => {
            println!("Stopping playback");
            println!("\nNote: Use 'storystream tui' for full interactive experience");
        }
        Commands::List { author, favorites } => {
            println!("Listing audiobooks:");
            if let Some(a) = author {
                println!("  Filtered by author: {}", a);
            }
            if favorites {
                println!("  Showing favorites only");
            }
            println!("\nNote: Use 'storystream tui' for full interactive library browser");
        }
        Commands::Scan { path } => {
            if let Some(p) = path {
                println!("Scanning path: {}", p);
            } else {
                println!("Scanning configured library paths");
            }
            println!("\nNote: Use 'storystream tui' for full interactive experience");
        }
        Commands::Search { query } => {
            println!("Searching for: {}", query);
            println!("\nNote: Use 'storystream tui' for full interactive search");
        }
        Commands::Bookmark { title } => {
            if let Some(t) = title {
                println!("Adding bookmark: {}", t);
            } else {
                println!("Adding bookmark at current position");
            }
            println!("\nNote: Use 'storystream tui' for full bookmark management");
        }
        Commands::Status => {
            println!("Current Status:");
            println!("  Playback: Stopped");
            println!("  Position: 00:00:00 / 00:00:00");
            println!("\nNote: Use 'storystream tui' for real-time status display");
        }
        Commands::Config { full } => {
            println!("StoryStream Configuration:");
            if full {
                println!("  (Full configuration would be displayed here)");
            } else {
                println!("  Database: ~/.local/share/storystream/storystream.db");
                println!("  Config: ~/.config/storystream/config.toml");
                println!("\nUse --full to see complete configuration");
            }
        }
    }

    Ok(())
}
