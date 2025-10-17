// crates/cli/src/commands.rs
//! Command-line interface definitions

use clap::{Parser, Subcommand};

/// StoryStream CLI application
#[derive(Parser)]
#[command(name = "storystream")]
#[command(author = "StoryStream Contributors")]
#[command(version = "1.0.0")]
#[command(about = "Professional Audiobook Player", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Play an audiobook
    Play {
        /// Title or path of the audiobook
        book: String,

        /// Playback speed (0.5 - 3.0)
        #[arg(short, long)]
        speed: Option<f32>,

        /// Volume (0 - 100)
        #[arg(short, long)]
        volume: Option<u8>,
    },

    /// Pause current playback
    Pause,

    /// Resume paused playback
    Resume,

    /// Stop playback
    Stop,

    /// List all audiobooks in library
    List {
        /// Filter by author
        #[arg(short, long)]
        author: Option<String>,

        /// Show only favorites
        #[arg(short, long)]
        favorites: bool,
    },

    /// Scan library for new audiobooks
    Scan {
        /// Path to scan (uses config paths if not specified)
        path: Option<String>,
    },

    /// Search for audiobooks
    Search {
        /// Search query
        query: String,
    },

    /// Add a bookmark at current position
    Bookmark {
        /// Optional bookmark title
        #[arg(short, long)]
        title: Option<String>,
    },

    /// Show current playback status
    Status,

    /// Launch the terminal user interface
    Tui,

    /// Show application configuration
    Config {
        /// Show full configuration
        #[arg(short, long)]
        full: bool,
    },
}