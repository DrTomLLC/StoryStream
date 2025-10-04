use anyhow::{Context, Result};
use storystream_database::{connection::{connect, DatabaseConfig}, queries::books, search::search_books};
use storystream_core::BookId;
use console::style;
use std::path::Path;

pub async fn list_books(db_path: &str) -> Result<()> {
    let config = DatabaseConfig::new(db_path);
    let pool = connect(config).await?;
    let books = books::list_books(&pool).await?;

    if books.is_empty() {
        println!("📚 No audiobooks in library");
        println!("\nImport a book with: storystream import <path>");
        return Ok(());
    }

    println!("\n📚 {} Audiobook(s)\n", books.len());
    println!("{:<5} {:<40} {:<30} {:>10}", "ID", "Title", "Author", "Duration");
    println!("{}", "─".repeat(90));

    for book in books {
        let duration = format_duration(book.duration.as_seconds());
        let author = book.author.as_deref().unwrap_or("Unknown");

        println!(
            "{:<5} {:<40} {:<30} {:>10}",
            book.id.to_string(),
            truncate(&book.title, 40),
            truncate(author, 30),
            duration
        );
    }

    println!();
    Ok(())
}

pub async fn search_books_cmd(db_path: &str, query: &str) -> Result<()> {
    let config = DatabaseConfig::new(db_path);
    let pool = connect(config).await?;

    println!("🔍 Searching for '{}'...\n", style(query).cyan());

    let results = search_books(&pool, query, 20).await?;

    if results.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    println!("Found {} result(s):\n", results.len());

    for result in results {
        println!("  {} {}", style("▶").green(), style(&result.item.title).bold());
        if let Some(author) = &result.item.author {
            println!("    by {}", style(author).dim());
        }
        println!("    ID: {}", result.item.id);
        println!();
    }

    Ok(())
}

pub async fn play_book(db_path: &str, book_id: &str) -> Result<()> {
    let config = DatabaseConfig::new(db_path);
    let pool = connect(config).await?;

    // Parse BookId from string
    let book_id = BookId::from_string(book_id)
        .context("Invalid book ID format")?;

    // Get book info
    let book = books::get_book(&pool, book_id).await?;

    println!("\n🎵 Loading: {}", style(&book.title).bold().cyan());
    if let Some(author) = &book.author {
        println!("   by {}\n", style(author).dim());
    }

    // Start player
    crate::player::start_playback(db_path, book).await?;

    Ok(())
}

pub async fn import_book(
    db_path: &str,
    path: &str,
    title: Option<&str>,
    author: Option<&str>,
) -> Result<()> {
    let config = DatabaseConfig::new(db_path);
    let pool = connect(config).await?;

    // Check if file exists
    if !Path::new(path).exists() {
        anyhow::bail!("File not found: {}", path);
    }

    // Use filename as title if not provided
    let title = title
        .map(String::from)
        .or_else(|| {
            Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .map(String::from)
        })
        .context("Could not determine title")?;

    println!("📥 Importing: {}", style(&title).cyan());

    // Create book using core types
    use storystream_core::{Book, Duration};
    use std::path::PathBuf;

    let mut book = Book::new(
        title.clone(),
        PathBuf::from(path),
        0, // file size - would need to calculate
        Duration::from_seconds(0), // duration - would need to detect
    );

    if let Some(author) = author {
        book.author = Some(author.to_string());
    }

    books::create_book(&pool, &book).await?;

    println!("✓ Book imported with ID: {}", style(book.id).green());
    println!("\nPlay with: storystream play {}", book.id);

    Ok(())
}

pub async fn show_info(db_path: &str, book_id: &str) -> Result<()> {
    let config = DatabaseConfig::new(db_path);
    let pool = connect(config).await?;

    // Parse BookId from string
    let book_id = BookId::from_string(book_id)
        .context("Invalid book ID format")?;

    let book = books::get_book(&pool, book_id).await?;

    println!("\n📖 Book Information\n");
    println!("  Title: {}", style(&book.title).bold());

    if let Some(author) = &book.author {
        println!("  Author: {}", author);
    }

    println!("  File: {}", book.file_path.display());
    println!("  Duration: {}", format_duration(book.duration.as_seconds()));

    println!();
    Ok(())
}

pub(crate) fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

pub(crate) fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {:02}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
#[cfg(test)]
mod tests;