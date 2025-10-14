// FILE: crates/cli/src/main.rs

use anyhow::{Context, Result};
use clap::{Arg, Command};

mod commands;
mod player;

fn build_cli() -> Command {
    Command::new("storystream")
        .version("0.1.0")
        .author("StoryStream Team")
        .about("Cross-platform audiobook player and library manager")
        .arg(
            Arg::new("database")
                .short('d')
                .long("database")
                .value_name("PATH")
                .help("Path to the database file")
                .default_value("storystream.db")
                .global(true),
        )
        .subcommand(Command::new("init").about("Initialize the database and create tables"))
        .subcommand(
            Command::new("list")
                .about("List all books in the library")
                .arg(
                    Arg::new("favorites")
                        .short('f')
                        .long("favorites")
                        .help("Show only favorite books")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("add")
                .about("Add a new book to the library")
                .arg(Arg::new("path").required(true).value_name("FILE").help("Path to the audiobook file"))
                .arg(Arg::new("title").short('t').long("title").value_name("TITLE").help("Book title (optional)"))
                .arg(Arg::new("author").short('a').long("author").value_name("AUTHOR").help("Book author (optional)")),
        )
        .subcommand(
            Command::new("search")
                .about("Search for books")
                .arg(Arg::new("query").required(true).value_name("QUERY").help("Search query")),
        )
        .subcommand(
            Command::new("info")
                .about("Show detailed information about a book")
                .arg(Arg::new("id").required(true).value_name("BOOK_ID").help("Book ID (UUID)")),
        )
        .subcommand(
            Command::new("play")
                .about("Play an audiobook")
                .arg(Arg::new("id").required(true).value_name("BOOK_ID").help("Book ID (UUID) to play")),
        )
        .subcommand(
            Command::new("delete")
                .about("Delete a book from the library")
                .arg(Arg::new("id").required(true).value_name("BOOK_ID").help("Book ID (UUID) to delete"))
                .arg(Arg::new("force").short('f').long("force").help("Skip confirmation prompt").action(clap::ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("favorite")
                .about("Mark a book as favorite or remove from favorites")
                .arg(Arg::new("id").required(true).value_name("BOOK_ID").help("Book ID (UUID)"))
                .arg(Arg::new("remove").short('r').long("remove").help("Remove from favorites").action(clap::ArgAction::SetTrue)),
        )
        .subcommand(Command::new("stats").about("Show library statistics"))
        .subcommand(
            Command::new("export")
                .about("Export library data")
                .arg(Arg::new("output").short('o').long("output").value_name("FILE").help("Output file path").default_value("library_export.json"))
                .arg(Arg::new("format").short('f').long("format").value_name("FORMAT").help("Export format").value_parser(["json", "csv"]).default_value("json")),
        )
}

async fn ensure_database_ready(db_path: &str) -> Result<()> {
    use storystream_database::{connection::{connect, DatabaseConfig}, migrations::run_migrations};
    let config = DatabaseConfig::new(db_path);
    let pool = connect(config).await.context("Failed to connect to database")?;
    run_migrations(&pool).await.context("Failed to apply database migrations")?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let matches = build_cli().get_matches();
    let db_path = matches.get_one::<String>("database").map(|s| s.as_str()).unwrap_or("storystream.db");
    ensure_database_ready(db_path).await.context("Failed to initialize database")?;

    match matches.subcommand() {
        Some(("init", _)) => {
            println!("Database initialized at {}", db_path);
            Ok(())
        }
        Some(("list", _)) => commands::list_books(db_path).await,
        Some(("add", sub_matches)) => commands::add_book(db_path, sub_matches).await,
        Some(("search", sub_matches)) => commands::search_books(db_path, sub_matches).await,
        Some(("info", sub_matches)) => commands::show_book_info(db_path, sub_matches).await,
        Some(("play", sub_matches)) => {
            let book_id = sub_matches.get_one::<String>("id").ok_or_else(|| anyhow::anyhow!("Book ID is required"))?;
            commands::play_book(db_path, book_id).await
        }
        Some(("delete", sub_matches)) => commands::delete_book(db_path, sub_matches).await,
        Some(("favorite", sub_matches)) => commands::toggle_favorite(db_path, sub_matches).await,
        Some(("stats", _)) => commands::show_stats(db_path).await,
        Some(("export", sub_matches)) => commands::export_library(db_path, sub_matches).await,
        _ => {
            build_cli().print_help()?;
            Ok(())
        }
    }
}