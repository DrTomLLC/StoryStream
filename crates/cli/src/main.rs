mod commands;
mod player;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "storystream")]
#[command(about = "StoryStream - Audiobook Player", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Database path
    #[arg(short, long, default_value = "storystream.db")]
    db: String,
}

#[derive(Subcommand)]
enum Commands {
    /// List all audiobooks
    List,

    /// Search for audiobooks
    Search {
        /// Search query
        query: String,
    },

    /// Play an audiobook
    Play {
        /// Book ID to play
        book_id: String,
    },

    /// Import an audiobook
    Import {
        /// Path to audio file
        path: String,

        /// Book title
        #[arg(short, long)]
        title: Option<String>,

        /// Author name
        #[arg(short, long)]
        author: Option<String>,
    },

    /// Show book details
    Info {
        /// Book ID
        book_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => commands::list_books(&cli.db).await?,
        Commands::Search { query } => commands::search_books_cmd(&cli.db, &query).await?,
        Commands::Play { book_id } => commands::play_book(&cli.db, &book_id).await?,
        Commands::Import { path, title, author } => {
            commands::import_book(&cli.db, &path, title.as_deref(), author.as_deref()).await?
        }
        Commands::Info { book_id } => commands::show_info(&cli.db, &book_id).await?,
    }

    Ok(())
}