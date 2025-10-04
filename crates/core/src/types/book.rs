//! Book and chapter domain models

use crate::types::{Duration, Timestamp, Validator};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a book
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BookId(Uuid);

impl BookId {
    /// Creates a new random BookId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a BookId from a UUID string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Returns the BookId as a string
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for BookId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BookId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a chapter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChapterId(Uuid);

impl ChapterId {
    /// Creates a new random ChapterId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a ChapterId from a UUID string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Returns the ChapterId as a string
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for ChapterId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ChapterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a complete audiobook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: BookId,
    pub title: String,
    pub author: Option<String>,
    pub narrator: Option<String>,
    pub series: Option<String>,
    pub series_position: Option<f32>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub duration: Duration,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub cover_art_path: Option<PathBuf>,
    pub added_date: Timestamp,
    pub last_played: Option<Timestamp>,
    pub play_count: u32,
    pub is_favorite: bool,
    pub rating: Option<u8>, // 1-5 stars
    pub tags: Vec<String>,
    pub deleted_at: Option<Timestamp>, // Soft delete
}

impl Book {
    /// Creates a new book with required fields
    pub fn new(
        title: String,
        file_path: PathBuf,
        file_size: u64,
        duration: Duration,
    ) -> Self {
        Self {
            id: BookId::new(),
            title,
            author: None,
            narrator: None,
            series: None,
            series_position: None,
            description: None,
            language: None,
            publisher: None,
            published_date: None,
            isbn: None,
            duration,
            file_path,
            file_size,
            cover_art_path: None,
            added_date: Timestamp::now(),
            last_played: None,
            play_count: 0,
            is_favorite: false,
            rating: None,
            tags: Vec::new(),
            deleted_at: None,
        }
    }

    /// Returns true if this book is deleted (soft delete)
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Marks the book as deleted
    pub fn delete(&mut self) {
        self.deleted_at = Some(Timestamp::now());
    }

    /// Restores a deleted book
    pub fn restore(&mut self) {
        self.deleted_at = None;
    }

    /// Increments the play count and updates last played
    pub fn mark_played(&mut self) {
        self.play_count += 1;
        self.last_played = Some(Timestamp::now());
    }
}

impl Validator for Book {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.title.trim().is_empty() {
            errors.push("Title cannot be empty".to_string());
        }

        if self.duration.is_zero() {
            errors.push("Duration must be greater than zero".to_string());
        }

        if self.file_size == 0 {
            errors.push("File size must be greater than zero".to_string());
        }

        if let Some(rating) = self.rating {
            if !(1..=5).contains(&rating) {
                errors.push("Rating must be between 1 and 5".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Represents a chapter within an audiobook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: ChapterId,
    pub book_id: BookId,
    pub title: String,
    pub index: u32,
    pub start_time: Duration,
    pub end_time: Duration,
    pub image_path: Option<PathBuf>,
}

impl Chapter {
    /// Creates a new chapter
    pub fn new(
        book_id: BookId,
        title: String,
        index: u32,
        start_time: Duration,
        end_time: Duration,
    ) -> Self {
        Self {
            id: ChapterId::new(),
            book_id,
            title,
            index,
            start_time,
            end_time,
            image_path: None,
        }
    }

    /// Returns the duration of this chapter
    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.end_time.as_millis() - self.start_time.as_millis())
    }
}

impl Validator for Chapter {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.title.trim().is_empty() {
            errors.push("Chapter title cannot be empty".to_string());
        }

        if self.end_time <= self.start_time {
            errors.push("End time must be after start time".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_id_creation() {
        let id1 = BookId::new();
        let id2 = BookId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_book_id_from_string() {
        let id = BookId::new();
        let s = id.as_string();
        let parsed = BookId::from_string(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_book_id_display() {
        let id = BookId::new();
        let display = format!("{}", id);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_chapter_id_creation() {
        let id1 = ChapterId::new();
        let id2 = ChapterId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_book_new() {
        let book = Book::new(
            "Test Book".to_string(),
            PathBuf::from("/test/book.mp3"),
            1000000,
            Duration::from_seconds(3600),
        );

        assert_eq!(book.title, "Test Book");
        assert_eq!(book.duration.as_seconds(), 3600);
        assert_eq!(book.play_count, 0);
        assert!(!book.is_favorite);
        assert!(!book.is_deleted());
    }

    #[test]
    fn test_book_validation_success() {
        let book = Book::new(
            "Valid Book".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        assert!(book.is_valid());
    }

    #[test]
    fn test_book_validation_empty_title() {
        let mut book = Book::new(
            "   ".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        book.title = "   ".to_string();
        assert!(!book.is_valid());
    }

    #[test]
    fn test_book_validation_zero_duration() {
        let book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_millis(0),
        );
        assert!(!book.is_valid());
    }

    #[test]
    fn test_book_validation_invalid_rating() {
        let mut book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );
        book.rating = Some(6);
        assert!(!book.is_valid());
    }

    #[test]
    fn test_book_soft_delete() {
        let mut book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );

        assert!(!book.is_deleted());
        book.delete();
        assert!(book.is_deleted());
        book.restore();
        assert!(!book.is_deleted());
    }

    #[test]
    fn test_book_mark_played() {
        let mut book = Book::new(
            "Test".to_string(),
            PathBuf::from("/test.mp3"),
            1000,
            Duration::from_seconds(100),
        );

        assert_eq!(book.play_count, 0);
        assert!(book.last_played.is_none());

        book.mark_played();

        assert_eq!(book.play_count, 1);
        assert!(book.last_played.is_some());

        book.mark_played();
        assert_eq!(book.play_count, 2);
    }

    #[test]
    fn test_chapter_new() {
        let book_id = BookId::new();
        let chapter = Chapter::new(
            book_id,
            "Chapter 1".to_string(),
            1,
            Duration::from_seconds(0),
            Duration::from_seconds(600),
        );

        assert_eq!(chapter.book_id, book_id);
        assert_eq!(chapter.title, "Chapter 1");
        assert_eq!(chapter.index, 1);
    }

    #[test]
    fn test_chapter_duration() {
        let chapter = Chapter::new(
            BookId::new(),
            "Test".to_string(),
            1,
            Duration::from_seconds(100),
            Duration::from_seconds(200),
        );

        assert_eq!(chapter.duration().as_seconds(), 100);
    }

    #[test]
    fn test_chapter_validation_success() {
        let chapter = Chapter::new(
            BookId::new(),
            "Valid Chapter".to_string(),
            1,
            Duration::from_seconds(0),
            Duration::from_seconds(100),
        );
        assert!(chapter.is_valid());
    }

    #[test]
    fn test_chapter_validation_empty_title() {
        let mut chapter = Chapter::new(
            BookId::new(),
            "Test".to_string(),
            1,
            Duration::from_seconds(0),
            Duration::from_seconds(100),
        );
        chapter.title = "   ".to_string();
        assert!(!chapter.is_valid());
    }

    #[test]
    fn test_chapter_validation_invalid_times() {
        let chapter = Chapter::new(
            BookId::new(),
            "Test".to_string(),
            1,
            Duration::from_seconds(100),
            Duration::from_seconds(50),
        );
        assert!(!chapter.is_valid());
    }
}