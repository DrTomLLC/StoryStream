//! Playlist domain models

use crate::types::{BookId, Timestamp, Validator};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a playlist
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaylistId(Uuid);

impl PlaylistId {
    /// Creates a new random PlaylistId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a PlaylistId from a UUID string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Returns the PlaylistId as a string
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for PlaylistId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PlaylistId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of playlist
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaylistType {
    /// Manually curated playlist
    Manual,
    /// Smart playlist with automatic criteria
    Smart,
}

/// Represents a playlist of audiobooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub description: Option<String>,
    pub playlist_type: PlaylistType,
    pub smart_criteria: Option<SmartPlaylistCriteria>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Playlist {
    /// Creates a new manual playlist
    pub fn new_manual(name: String) -> Self {
        let now = Timestamp::now();
        Self {
            id: PlaylistId::new(),
            name,
            description: None,
            playlist_type: PlaylistType::Manual,
            smart_criteria: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new smart playlist
    pub fn new_smart(name: String, criteria: SmartPlaylistCriteria) -> Self {
        let now = Timestamp::now();
        Self {
            id: PlaylistId::new(),
            name,
            description: None,
            playlist_type: PlaylistType::Smart,
            smart_criteria: Some(criteria),
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns true if this is a smart playlist
    pub fn is_smart(&self) -> bool {
        self.playlist_type == PlaylistType::Smart
    }

    /// Updates the playlist name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Timestamp::now();
    }

    /// Updates the playlist description
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Timestamp::now();
    }
}

impl Validator for Playlist {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push("Playlist name cannot be empty".to_string());
        }

        if self.playlist_type == PlaylistType::Smart && self.smart_criteria.is_none() {
            errors.push("Smart playlist must have criteria".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Criteria for smart playlists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartPlaylistCriteria {
    pub favorite_only: bool,
    pub unfinished_only: bool,
    pub min_rating: Option<u8>,
    pub authors: Vec<String>,
    pub narrators: Vec<String>,
    pub tags: Vec<String>,
    pub series: Vec<String>,
    pub max_results: Option<usize>,
}

impl SmartPlaylistCriteria {
    /// Creates criteria for favorite books
    pub fn favorites() -> Self {
        Self {
            favorite_only: true,
            unfinished_only: false,
            min_rating: None,
            authors: Vec::new(),
            narrators: Vec::new(),
            tags: Vec::new(),
            series: Vec::new(),
            max_results: None,
        }
    }

    /// Creates criteria for unfinished books
    pub fn unfinished() -> Self {
        Self {
            favorite_only: false,
            unfinished_only: true,
            min_rating: None,
            authors: Vec::new(),
            narrators: Vec::new(),
            tags: Vec::new(),
            series: Vec::new(),
            max_results: None,
        }
    }

    /// Creates criteria for books by specific authors
    pub fn by_authors(authors: Vec<String>) -> Self {
        Self {
            favorite_only: false,
            unfinished_only: false,
            min_rating: None,
            authors,
            narrators: Vec::new(),
            tags: Vec::new(),
            series: Vec::new(),
            max_results: None,
        }
    }

    /// Creates criteria for highly rated books
    pub fn highly_rated(min_rating: u8) -> Self {
        Self {
            favorite_only: false,
            unfinished_only: false,
            min_rating: Some(min_rating),
            authors: Vec::new(),
            narrators: Vec::new(),
            tags: Vec::new(),
            series: Vec::new(),
            max_results: None,
        }
    }
}

impl Default for SmartPlaylistCriteria {
    fn default() -> Self {
        Self {
            favorite_only: false,
            unfinished_only: false,
            min_rating: None,
            authors: Vec::new(),
            narrators: Vec::new(),
            tags: Vec::new(),
            series: Vec::new(),
            max_results: Some(100),
        }
    }
}

/// Represents an item in a playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub playlist_id: PlaylistId,
    pub book_id: BookId,
    pub position: u32,
    pub added_at: Timestamp,
}

impl PlaylistItem {
    /// Creates a new playlist item
    pub fn new(playlist_id: PlaylistId, book_id: BookId, position: u32) -> Self {
        Self {
            playlist_id,
            book_id,
            position,
            added_at: Timestamp::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_id_creation() {
        let id1 = PlaylistId::new();
        let id2 = PlaylistId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_playlist_id_from_string() {
        let id = PlaylistId::new();
        let s = id.as_string();
        let parsed = PlaylistId::from_string(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_playlist_id_display() {
        let id = PlaylistId::new();
        let display = format!("{}", id);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_playlist_new_manual() {
        let playlist = Playlist::new_manual("My Playlist".to_string());

        assert_eq!(playlist.name, "My Playlist");
        assert_eq!(playlist.playlist_type, PlaylistType::Manual);
        assert!(!playlist.is_smart());
        assert!(playlist.smart_criteria.is_none());
    }

    #[test]
    fn test_playlist_new_smart() {
        let criteria = SmartPlaylistCriteria::favorites();
        let playlist = Playlist::new_smart("Favorites".to_string(), criteria);

        assert_eq!(playlist.name, "Favorites");
        assert_eq!(playlist.playlist_type, PlaylistType::Smart);
        assert!(playlist.is_smart());
        assert!(playlist.smart_criteria.is_some());
    }

    #[test]
    fn test_playlist_set_name() {
        let mut playlist = Playlist::new_manual("Old Name".to_string());
        let before = playlist.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        playlist.set_name("New Name".to_string());

        assert_eq!(playlist.name, "New Name");
        assert!(playlist.updated_at > before);
    }

    #[test]
    fn test_playlist_set_description() {
        let mut playlist = Playlist::new_manual("Test".to_string());

        playlist.set_description(Some("A description".to_string()));
        assert_eq!(playlist.description, Some("A description".to_string()));
    }

    #[test]
    fn test_playlist_validation_success() {
        let playlist = Playlist::new_manual("Valid".to_string());
        assert!(playlist.is_valid());
    }

    #[test]
    fn test_playlist_validation_empty_name() {
        let mut playlist = Playlist::new_manual("Test".to_string());
        playlist.name = "   ".to_string();
        assert!(!playlist.is_valid());
    }

    #[test]
    fn test_playlist_validation_smart_without_criteria() {
        let mut playlist = Playlist::new_manual("Test".to_string());
        playlist.playlist_type = PlaylistType::Smart;
        playlist.smart_criteria = None;
        assert!(!playlist.is_valid());
    }

    #[test]
    fn test_smart_criteria_favorites() {
        let criteria = SmartPlaylistCriteria::favorites();
        assert!(criteria.favorite_only);
        assert!(!criteria.unfinished_only);
    }

    #[test]
    fn test_smart_criteria_unfinished() {
        let criteria = SmartPlaylistCriteria::unfinished();
        assert!(!criteria.favorite_only);
        assert!(criteria.unfinished_only);
    }

    #[test]
    fn test_smart_criteria_by_authors() {
        let authors = vec!["Author 1".to_string(), "Author 2".to_string()];
        let criteria = SmartPlaylistCriteria::by_authors(authors.clone());
        assert_eq!(criteria.authors, authors);
    }

    #[test]
    fn test_smart_criteria_highly_rated() {
        let criteria = SmartPlaylistCriteria::highly_rated(4);
        assert_eq!(criteria.min_rating, Some(4));
    }

    #[test]
    fn test_smart_criteria_default() {
        let criteria = SmartPlaylistCriteria::default();
        assert!(!criteria.favorite_only);
        assert!(!criteria.unfinished_only);
        assert_eq!(criteria.max_results, Some(100));
    }

    #[test]
    fn test_playlist_item_new() {
        let playlist_id = PlaylistId::new();
        let book_id = BookId::new();
        let item = PlaylistItem::new(playlist_id, book_id, 0);

        assert_eq!(item.playlist_id, playlist_id);
        assert_eq!(item.book_id, book_id);
        assert_eq!(item.position, 0);
    }

    #[test]
    fn test_playlist_item_ordering() {
        let playlist_id = PlaylistId::new();
        let item1 = PlaylistItem::new(playlist_id, BookId::new(), 0);
        let item2 = PlaylistItem::new(playlist_id, BookId::new(), 1);
        let item3 = PlaylistItem::new(playlist_id, BookId::new(), 2);

        assert!(item1.position < item2.position);
        assert!(item2.position < item3.position);
    }
}