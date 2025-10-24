//! Bookmark domain model

use crate::types::{BookId, Duration, Timestamp, Validator};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a bookmark
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BookmarkId(Uuid);

impl BookmarkId {
    /// Creates a new random BookmarkId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a BookmarkId from a UUID string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Returns the BookmarkId as a string
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for BookmarkId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BookmarkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a user bookmark in an audiobook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: BookmarkId,
    pub book_id: BookId,
    pub position: Duration,
    pub title: Option<String>,
    pub note: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Bookmark {
    /// Creates a new bookmark at the specified position
    pub fn new(book_id: BookId, position: Duration) -> Self {
        let now = Timestamp::now();
        Self {
            id: BookmarkId::new(),
            book_id,
            position,
            title: None,
            note: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a bookmark with a title
    pub fn with_title(book_id: BookId, position: Duration, title: String) -> Self {
        let mut bookmark = Self::new(book_id, position);
        bookmark.title = Some(title);
        bookmark
    }

    /// Updates the bookmark's note
    pub fn set_note(&mut self, note: String) {
        self.note = Some(note);
        self.updated_at = Timestamp::now();
    }

    /// Updates the bookmark's title
    pub fn set_title(&mut self, title: String) {
        self.title = Some(title);
        self.updated_at = Timestamp::now();
    }

    /// Returns true if this bookmark has a note
    pub fn has_note(&self) -> bool {
        self.note.as_ref().map_or(false, |n| !n.trim().is_empty())
    }

    /// Returns true if this bookmark has a title
    pub fn has_title(&self) -> bool {
        self.title.as_ref().map_or(false, |t| !t.trim().is_empty())
    }
}

impl Validator for Bookmark {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.position.is_zero() {
            errors.push("Bookmark position must be greater than zero".to_string());
        }

        if let Some(title) = &self.title {
            if title.trim().is_empty() {
                errors.push("Bookmark title cannot be empty if set".to_string());
            }
        }

        if let Some(note) = &self.note {
            if note.trim().is_empty() {
                errors.push("Bookmark note cannot be empty if set".to_string());
            }
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
    fn test_bookmark_id_creation() {
        let id1 = BookmarkId::new();
        let id2 = BookmarkId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_bookmark_id_from_string() {
        let id = BookmarkId::new();
        let s = id.as_string();
        let parsed = BookmarkId::from_string(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_bookmark_id_display() {
        let id = BookmarkId::new();
        let display = format!("{}", id);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_bookmark_new() {
        let book_id = BookId::new();
        let position = Duration::from_seconds(1234);
        let bookmark = Bookmark::new(book_id, position);

        assert_eq!(bookmark.book_id, book_id);
        assert_eq!(bookmark.position, position);
        assert!(bookmark.title.is_none());
        assert!(bookmark.note.is_none());
    }

    #[test]
    fn test_bookmark_with_title() {
        let book_id = BookId::new();
        let position = Duration::from_seconds(1234);
        let bookmark = Bookmark::with_title(book_id, position, "Important moment".to_string());

        assert_eq!(bookmark.title, Some("Important moment".to_string()));
        assert!(bookmark.has_title());
    }

    #[test]
    fn test_bookmark_set_note() {
        let mut bookmark = Bookmark::new(BookId::new(), Duration::from_seconds(100));

        assert!(!bookmark.has_note());

        let before = bookmark.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));

        bookmark.set_note("This is a note".to_string());

        assert!(bookmark.has_note());
        assert_eq!(bookmark.note, Some("This is a note".to_string()));
        assert!(bookmark.updated_at > before);
    }

    #[test]
    fn test_bookmark_set_title() {
        let mut bookmark = Bookmark::new(BookId::new(), Duration::from_seconds(100));

        assert!(!bookmark.has_title());

        let before = bookmark.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));

        bookmark.set_title("New Title".to_string());

        assert!(bookmark.has_title());
        assert_eq!(bookmark.title, Some("New Title".to_string()));
        assert!(bookmark.updated_at > before);
    }

    #[test]
    fn test_bookmark_has_note_empty_string() {
        let mut bookmark = Bookmark::new(BookId::new(), Duration::from_seconds(100));
        bookmark.note = Some("   ".to_string());
        assert!(!bookmark.has_note());
    }

    #[test]
    fn test_bookmark_has_title_empty_string() {
        let mut bookmark = Bookmark::new(BookId::new(), Duration::from_seconds(100));
        bookmark.title = Some("   ".to_string());
        assert!(!bookmark.has_title());
    }

    #[test]
    fn test_bookmark_validation_success() {
        let bookmark = Bookmark::new(BookId::new(), Duration::from_seconds(100));
        assert!(bookmark.is_valid());
    }

    #[test]
    fn test_bookmark_validation_zero_position() {
        let bookmark = Bookmark::new(BookId::new(), Duration::from_millis(0));
        assert!(!bookmark.is_valid());
    }

    #[test]
    fn test_bookmark_validation_empty_title() {
        let mut bookmark = Bookmark::new(BookId::new(), Duration::from_seconds(100));
        bookmark.title = Some("   ".to_string());
        assert!(!bookmark.is_valid());
    }

    #[test]
    fn test_bookmark_validation_empty_note() {
        let mut bookmark = Bookmark::new(BookId::new(), Duration::from_seconds(100));
        bookmark.note = Some("   ".to_string());
        assert!(!bookmark.is_valid());
    }

    #[test]
    fn test_bookmark_timestamps() {
        let bookmark = Bookmark::new(BookId::new(), Duration::from_seconds(100));
        assert_eq!(bookmark.created_at, bookmark.updated_at);
    }
}
