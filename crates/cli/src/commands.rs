// FILE: crates/cli/src/commands.rs

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use console::style;
use storystream_core::{Book, BookId, Duration as CoreDuration};
use storystream_database::{
    connection::{connect, DatabaseConfig},
    queries::books,
    search::search_books as db_search_books,
    DbPool,
};
use std::path::PathBuf;

/// List all books in the library
pub async fn list_books(db_path: &str) -> Result<()> {
    let pool = connect_db(db_path).await?;
    let books = books::list_books(&pool)
        .await
        .context("Failed to list books")?;

    if books.is_empty() {
        println!("No books in library. Use 'add' command to import audiobooks.");
        return Ok(());
    }

    println!("\n{} Books in Library", style(books.len()).bold().cyan());
    println!("{}", "=".repeat(80));

    for book in books {
        print_book_summary(&book);
    }

    Ok(())
}

/// Add a new book to the library
pub async fn add_book(db_path: &str, matches: &ArgMatches) -> Result<()> {
    let file_path = matches
        .get_one::<String>("path")
        .ok_or_else(|| anyhow::anyhow!("File path is required"))?;

    let pool = connect_db(db_path).await?;

    let path = PathBuf::from(file_path);
    if !path.exists() {
        bail!("File not found: {}", file_path);
    }

    let metadata = std::fs::metadata(&path)
        .context("Failed to read file metadata")?;
    let file_size = metadata.len();

    let title = matches
        .get_one::<String>("title")
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });

    let author = matches
        .get_one::<String>("author")
        .map(|s| s.to_string());

    let mut book = Book::new(
        title.clone(),
        path.clone(),
        file_size,
        CoreDuration::from_seconds(3600),
    );
    book.author = author;

    books::create_book(&pool, &book)
        .await
        .context("Failed to add book to database")?;

    println!("{} Book added successfully!", style("✓").green().bold());
    println!("  ID: {}", book.id);
    println!("  Title: {}", book.title);
    if let Some(author) = &book.author {
        println!("  Author: {}", author);
    }
    println!("  File: {}", file_path);

    Ok(())
}

/// Search for books
pub async fn search_books(db_path: &str, matches: &ArgMatches) -> Result<()> {
    let query = matches
        .get_one::<String>("query")
        .ok_or_else(|| anyhow::anyhow!("Search query is required"))?;

    let pool = connect_db(db_path).await?;
    let results = db_search_books(&pool, query, 50)
        .await
        .context("Failed to search books")?;

    if results.is_empty() {
        println!("No books found matching '{}'", query);
        return Ok(());
    }

    println!("\n{} Search Results for '{}'", style(results.len()).bold().cyan(), query);
    println!("{}", "=".repeat(80));

    for result in results {
        print_book_summary(&result.item);
        println!("  Rank: {}", result.rank);
        println!();
    }

    Ok(())
}

/// Show detailed information about a book
pub async fn show_book_info(db_path: &str, matches: &ArgMatches) -> Result<()> {
    let id_str = matches
        .get_one::<String>("id")
        .ok_or_else(|| anyhow::anyhow!("Book ID is required"))?;

    let book_id = BookId::from_string(id_str)
        .context("Invalid book ID format")?;

    let pool = connect_db(db_path).await?;
    let book = books::get_book(&pool, book_id)
        .await
        .context("Failed to get book")?;

    println!("\n{}", style("Book Information").bold().cyan());
    println!("{}", "=".repeat(80));
    println!("ID: {}", book.id);
    println!("Title: {}", style(&book.title).bold());

    if let Some(author) = &book.author {
        println!("Author: {}", author);
    }
    if let Some(narrator) = &book.narrator {
        println!("Narrator: {}", narrator);
    }
    if let Some(series) = &book.series {
        print!("Series: {}", series);
        if let Some(pos) = book.series_position {
            print!(" (Book {})", pos);
        }
        println!();
    }
    if let Some(description) = &book.description {
        println!("\nDescription:\n{}", description);
    }

    println!("\nFile Information:");
    println!("  Path: {}", book.file_path.display());
    println!("  Size: {}", format_size(book.file_size));
    println!("  Duration: {}", book.duration.as_hms());

    println!("\nPlayback Information:");
    println!("  Play Count: {}", book.play_count);
    if let Some(last_played) = book.last_played {
        println!("  Last Played: {}", last_played);
    }
    println!("  Favorite: {}", if book.is_favorite { "Yes" } else { "No" });
    if let Some(rating) = book.rating {
        println!("  Rating: {}/5", rating);
    }

    if !book.tags.is_empty() {
        println!("\nTags: {}", book.tags.join(", "));
    }

    Ok(())
}

/// Play an audiobook
pub async fn play_book(db_path: &str, book_id_str: &str) -> Result<()> {
    let book_id = BookId::from_string(book_id_str)
        .context("Invalid book ID format")?;

    let pool = connect_db(db_path).await?;
    let book = books::get_book(&pool, book_id)
        .await
        .context("Failed to get book")?;

    println!("\n{} {}", style("▶").green().bold(), style(&book.title).bold());
    if let Some(author) = &book.author {
        println!("by {}", author);
    }

    crate::player::start_playback(db_path, book).await?;

    Ok(())
}

/// Delete a book from the library
pub async fn delete_book(db_path: &str, matches: &ArgMatches) -> Result<()> {
    let id_str = matches
        .get_one::<String>("id")
        .ok_or_else(|| anyhow::anyhow!("Book ID is required"))?;

    let book_id = BookId::from_string(id_str)
        .context("Invalid book ID format")?;

    let force = matches.get_flag("force");

    let pool = connect_db(db_path).await?;
    let book = books::get_book(&pool, book_id)
        .await
        .context("Failed to get book")?;

    if !force {
        println!("Are you sure you want to delete '{}'? (y/N)", book.title);
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("Failed to read input")?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    books::delete_book(&pool, book_id)
        .await
        .context("Failed to delete book")?;

    println!("{} Book deleted: {}", style("✓").green().bold(), book.title);

    Ok(())
}

/// Toggle favorite status of a book
pub async fn toggle_favorite(db_path: &str, matches: &ArgMatches) -> Result<()> {
    let id_str = matches
        .get_one::<String>("id")
        .ok_or_else(|| anyhow::anyhow!("Book ID is required"))?;

    let book_id = BookId::from_string(id_str)
        .context("Invalid book ID format")?;

    let remove = matches.get_flag("remove");

    let pool = connect_db(db_path).await?;
    let mut book = books::get_book(&pool, book_id)
        .await
        .context("Failed to get book")?;

    if remove {
        book.is_favorite = false;
        println!("{} Removed '{}' from favorites", style("✓").green().bold(), book.title);
    } else {
        book.is_favorite = true;
        println!("{} Added '{}' to favorites", style("✓").green().bold(), book.title);
    }

    books::update_book(&pool, &book)
        .await
        .context("Failed to update book")?;

    Ok(())
}

/// Show library statistics
pub async fn show_stats(db_path: &str) -> Result<()> {
    let pool = connect_db(db_path).await?;
    let all_books = books::list_books(&pool)
        .await
        .context("Failed to list books")?;

    let total_count = all_books.len();
    let favorite_count = all_books.iter().filter(|b| b.is_favorite).count();
    let total_duration: u64 = all_books.iter().map(|b| b.duration.as_seconds()).sum();
    let total_size: u64 = all_books.iter().map(|b| b.file_size).sum();
    let avg_duration = if total_count > 0 { total_duration / total_count as u64 } else { 0 };

    println!("\n{}", style("Library Statistics").bold().cyan());
    println!("{}", "=".repeat(80));
    println!("Total Books: {}", style(total_count).bold());
    println!("Favorites: {}", style(favorite_count).bold());
    println!("Total Duration: {}", format_duration(total_duration));
    println!("Average Duration: {}", format_duration(avg_duration));
    println!("Total Size: {}", format_size(total_size));

    Ok(())
}

/// Export library data
pub async fn export_library(db_path: &str, matches: &ArgMatches) -> Result<()> {
    let output = matches
        .get_one::<String>("output")
        .map(|s| s.as_str())
        .unwrap_or("library_export.json");

    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("json");

    let pool = connect_db(db_path).await?;
    let books = books::list_books(&pool)
        .await
        .context("Failed to list books")?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&books)
                .context("Failed to serialize to JSON")?;
            std::fs::write(output, json)
                .context("Failed to write export file")?;
        }
        "csv" => {
            bail!("CSV export not yet implemented");
        }
        _ => {
            bail!("Unsupported format: {}", format);
        }
    }

    println!("{} Exported {} books to {}", style("✓").green().bold(), books.len(), output);

    Ok(())
}

async fn connect_db(db_path: &str) -> Result<DbPool> {
    let config = DatabaseConfig::new(db_path);
    connect(config)
        .await
        .context("Failed to connect to database")
}

fn print_book_summary(book: &Book) {
    println!("\n{}", style(&book.title).bold());
    if let Some(author) = &book.author {
        println!("  by {}", author);
    }
    println!("  ID: {} | Duration: {} | Size: {}",
             truncate(&book.id.to_string(), 8),
             book.duration.as_hms(),
             format_size(book.file_size)
    );
    if book.is_favorite {
        print!("  {}", style("★ Favorite").yellow());
    }
    if book.play_count > 0 {
        print!("  Played {} times", book.play_count);
    }
    println!();
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_format_duration() {
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(120), "2m");
        assert_eq!(format_duration(3600), "1h 0m");
    }

    #[tokio::test]
    async fn test_format_size() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1_048_576), "1.00 MB");
        assert_eq!(format_size(1_073_741_824), "1.00 GB");
    }

    #[tokio::test]
    async fn test_truncate() {
        assert_eq!(truncate("12345678", 8), "12345678");
        assert_eq!(truncate("123456789", 8), "12345678...");
    }
}