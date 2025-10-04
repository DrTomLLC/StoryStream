use super::*;
use storystream_core::{Book, Duration};
use storystream_database::{
    connection::{connect, DatabaseConfig},
    migrations::run_migrations,
    queries::books,
};
use std::path::PathBuf;
use tempfile::NamedTempFile;

async fn setup_test_db() -> (storystream_database::DbPool, NamedTempFile) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let config = DatabaseConfig::new(db_path);
    let pool = connect(config).await.unwrap();
    run_migrations(&pool).await.unwrap();

    (pool, temp_file)
}

async fn create_sample_book(pool: &storystream_database::DbPool, title: &str) -> Book {
    let mut book = Book::new(
        title.to_string(),
        PathBuf::from(format!("/test/{}.mp3", title)),
        1_000_000,
        Duration::from_seconds(3600),
    );
    book.author = Some("Test Author".to_string());
    books::create_book(pool, &book).await.unwrap();
    book
}

#[tokio::test]
async fn test_list_books_empty() {
    let (pool, _temp) = setup_test_db().await;
    let books = books::list_books(&pool).await.unwrap();
    assert_eq!(books.len(), 0);
}

#[tokio::test]
async fn test_list_books_with_data() {
    let (pool, _temp) = setup_test_db().await;

    create_sample_book(&pool, "Book One").await;
    create_sample_book(&pool, "Book Two").await;
    create_sample_book(&pool, "Book Three").await;

    let books = books::list_books(&pool).await.unwrap();
    assert_eq!(books.len(), 3);
}

#[tokio::test]
async fn test_search_books_finds_results() {
    let (pool, _temp) = setup_test_db().await;

    create_sample_book(&pool, "The Great Gatsby").await;
    create_sample_book(&pool, "Great Expectations").await;
    create_sample_book(&pool, "To Kill a Mockingbird").await;

    let results = storystream_database::search::search_books(&pool, "Great", 10)
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|r| r.item.title.contains("Gatsby")));
    assert!(results.iter().any(|r| r.item.title.contains("Expectations")));
}

#[tokio::test]
async fn test_search_books_no_results() {
    let (pool, _temp) = setup_test_db().await;

    create_sample_book(&pool, "Book One").await;

    let results = storystream_database::search::search_books(&pool, "NonexistentQuery", 10)
        .await
        .unwrap();

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_get_book_by_id() {
    let (pool, _temp) = setup_test_db().await;
    let book = create_sample_book(&pool, "Test Book").await;

    let retrieved = books::get_book(&pool, book.id).await.unwrap();

    assert_eq!(retrieved.id, book.id);
    assert_eq!(retrieved.title, "Test Book");
    assert_eq!(retrieved.author, Some("Test Author".to_string()));
}

#[tokio::test]
async fn test_get_book_invalid_id() {
    let (pool, _temp) = setup_test_db().await;

    let fake_id = storystream_core::BookId::new();
    let result = books::get_book(&pool, fake_id).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_book_id_parsing_valid() {
    let book_id = storystream_core::BookId::new();
    let id_string = book_id.as_string();

    let parsed = storystream_core::BookId::from_string(&id_string);
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), book_id);
}

#[tokio::test]
async fn test_book_id_parsing_invalid() {
    let result = storystream_core::BookId::from_string("not-a-valid-uuid");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_book_with_metadata() {
    let (pool, _temp) = setup_test_db().await;

    let mut book = Book::new(
        "Complete Book".to_string(),
        PathBuf::from("/test/complete.mp3"),
        5_000_000,
        Duration::from_seconds(7200),
    );
    book.author = Some("John Doe".to_string());
    book.narrator = Some("Jane Smith".to_string());
    book.series = Some("Test Series".to_string());
    book.series_position = Some(1.0);
    book.is_favorite = true;
    book.rating = Some(5);
    book.tags = vec!["fiction".to_string(), "classic".to_string()];

    books::create_book(&pool, &book).await.unwrap();

    let retrieved = books::get_book(&pool, book.id).await.unwrap();
    assert_eq!(retrieved.author, Some("John Doe".to_string()));
    assert_eq!(retrieved.narrator, Some("Jane Smith".to_string()));
    assert_eq!(retrieved.series, Some("Test Series".to_string()));
    assert_eq!(retrieved.series_position, Some(1.0));
    assert!(retrieved.is_favorite);
    assert_eq!(retrieved.rating, Some(5));
    assert_eq!(retrieved.tags.len(), 2);
}

#[tokio::test]
async fn test_update_book() {
    let (pool, _temp) = setup_test_db().await;
    let mut book = create_sample_book(&pool, "Original Title").await;

    book.title = "Updated Title".to_string();
    book.is_favorite = true;
    book.rating = Some(4);

    books::update_book(&pool, &book).await.unwrap();

    let retrieved = books::get_book(&pool, book.id).await.unwrap();
    assert_eq!(retrieved.title, "Updated Title");
    assert!(retrieved.is_favorite);
    assert_eq!(retrieved.rating, Some(4));
}

#[tokio::test]
async fn test_delete_book() {
    let (pool, _temp) = setup_test_db().await;
    let book = create_sample_book(&pool, "To Delete").await;

    books::delete_book(&pool, book.id).await.unwrap();

    let result = books::get_book(&pool, book.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_favorite_books() {
    let (pool, _temp) = setup_test_db().await;

    let mut book1 = create_sample_book(&pool, "Favorite 1").await;
    book1.is_favorite = true;
    books::update_book(&pool, &book1).await.unwrap();

    let mut book2 = create_sample_book(&pool, "Favorite 2").await;
    book2.is_favorite = true;
    books::update_book(&pool, &book2).await.unwrap();

    create_sample_book(&pool, "Not Favorite").await;

    let favorites = books::get_favorite_books(&pool).await.unwrap();
    assert_eq!(favorites.len(), 2);
    assert!(favorites.iter().all(|b| b.is_favorite));
}

#[tokio::test]
async fn test_truncate_function() {
    assert_eq!(truncate("short", 10), "short");
    assert_eq!(truncate("exactly ten!", 12), "exactly ten!");
    assert_eq!(truncate("this is a very long string", 10), "this is...");
}

#[tokio::test]
async fn test_format_duration() {
    assert_eq!(format_duration(30), "30s");
    assert_eq!(format_duration(90), "1m 30s");
    assert_eq!(format_duration(3665), "1h 01m 05s");
    assert_eq!(format_duration(7200), "2h 00m 00s");
}

#[tokio::test]
async fn test_database_connection_with_config() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let config = DatabaseConfig::new(db_path);
    let pool = connect(config).await.unwrap();

    run_migrations(&pool).await.unwrap();

    let books = books::list_books(&pool).await.unwrap();
    assert_eq!(books.len(), 0);
}

#[tokio::test]
async fn test_soft_delete_excluded_from_list() {
    let (pool, _temp) = setup_test_db().await;

    let mut book = create_sample_book(&pool, "To Soft Delete").await;
    book.delete();
    books::update_book(&pool, &book).await.unwrap();

    let all_books = books::list_books(&pool).await.unwrap();
    assert_eq!(all_books.len(), 0);

    // But can still retrieve directly
    let retrieved = books::get_book(&pool, book.id).await.unwrap();
    assert!(retrieved.is_deleted());
}

#[tokio::test]
async fn test_mark_played_updates_stats() {
    let (pool, _temp) = setup_test_db().await;
    let mut book = create_sample_book(&pool, "Test Book").await;

    assert_eq!(book.play_count, 0);
    assert!(book.last_played.is_none());

    book.mark_played();
    books::update_book(&pool, &book).await.unwrap();

    let retrieved = books::get_book(&pool, book.id).await.unwrap();
    assert_eq!(retrieved.play_count, 1);
    assert!(retrieved.last_played.is_some());
}