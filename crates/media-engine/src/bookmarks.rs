// crates/media-engine/src/bookmarks.rs
// NEW FILE - Complete bookmark management system

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Type of bookmark
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookmarkType {
    /// User-created bookmark
    User,
    /// Automatically created bookmark (e.g., on pause)
    Auto,
    /// Chapter start bookmark
    Chapter,
    /// Important moment marked by user
    Favorite,
}

/// A bookmark in an audiobook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Unique identifier
    pub id: String,
    /// Position in the audio file
    pub position: Duration,
    /// Type of bookmark
    pub bookmark_type: BookmarkType,
    /// Optional title/note
    pub title: Option<String>,
    /// Optional longer description
    pub note: Option<String>,
    /// When the bookmark was created
    pub created_at: SystemTime,
    /// When the bookmark was last accessed
    pub last_accessed: Option<SystemTime>,
    /// Number of times accessed
    pub access_count: u32,
    /// Optional chapter index if this is within a chapter
    pub chapter_index: Option<usize>,
}

impl Bookmark {
    /// Create a new bookmark
    pub fn new(position: Duration, bookmark_type: BookmarkType) -> Self {
        Self {
            id: Self::generate_id(),
            position,
            bookmark_type,
            title: None,
            note: None,
            created_at: SystemTime::now(),
            last_accessed: None,
            access_count: 0,
            chapter_index: None,
        }
    }

    /// Create a bookmark with a title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Create a bookmark with a note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Set the chapter index
    pub fn with_chapter(mut self, chapter_index: usize) -> Self {
        self.chapter_index = Some(chapter_index);
        self
    }

    /// Generate a unique ID
    fn generate_id() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("bm_{:x}", timestamp)
    }

    /// Mark bookmark as accessed
    pub fn mark_accessed(&mut self) {
        self.last_accessed = Some(SystemTime::now());
        self.access_count += 1;
    }

    /// Get a display string for the bookmark
    pub fn display_string(&self, total_duration: Duration) -> String {
        let pos_secs = self.position.as_secs();
        let hours = pos_secs / 3600;
        let minutes = (pos_secs % 3600) / 60;
        let seconds = pos_secs % 60;

        let time_str = if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        };

        let percent = if total_duration.as_secs() > 0 {
            (self.position.as_secs() as f32 / total_duration.as_secs() as f32 * 100.0) as u32
        } else {
            0
        };

        match &self.title {
            Some(title) => format!("{} - {} ({}%)", time_str, title, percent),
            None => format!("{} ({}%)", time_str, percent),
        }
    }
}

/// Manages bookmarks for an audiobook
pub struct BookmarkManager {
    /// All bookmarks, sorted by position
    bookmarks: BTreeMap<Duration, Bookmark>,
    /// Maximum number of auto-bookmarks to keep
    max_auto_bookmarks: usize,
    /// Current audiobook duration
    duration: Option<Duration>,
    /// Whether auto-bookmarking is enabled
    auto_bookmark_enabled: bool,
    /// Interval for auto-bookmarks
    auto_bookmark_interval: Duration,
    /// Last auto-bookmark position
    last_auto_position: Option<Duration>,
}

impl BookmarkManager {
    /// Create a new bookmark manager
    pub fn new() -> Self {
        Self {
            bookmarks: BTreeMap::new(),
            max_auto_bookmarks: 10,
            duration: None,
            auto_bookmark_enabled: true,
            auto_bookmark_interval: Duration::from_secs(300), // 5 minutes
            last_auto_position: None,
        }
    }

    /// Set the total duration of the audiobook
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = Some(duration);
    }

    /// Add a bookmark
    pub fn add_bookmark(&mut self, bookmark: Bookmark) -> Result<String, String> {
        // FIXED: Removed 'mut' from parameter as it's never mutated
        // Validate position
        if let Some(duration) = self.duration {
            if bookmark.position > duration {
                return Err("Bookmark position exceeds audiobook duration".to_string());
            }
        }

        // If it's an auto-bookmark, manage the limit
        if bookmark.bookmark_type == BookmarkType::Auto {
            self.cleanup_old_auto_bookmarks();
        }

        let id = bookmark.id.clone();
        self.bookmarks.insert(bookmark.position, bookmark);
        Ok(id)
    }

    /// Remove a bookmark by ID
    pub fn remove_bookmark(&mut self, bookmark_id: &str) -> Result<(), String> {
        let position = self
            .bookmarks
            .iter()
            .find(|(_, b)| b.id == bookmark_id)
            .map(|(pos, _)| *pos)
            .ok_or_else(|| format!("Bookmark not found: {}", bookmark_id))?;

        self.bookmarks.remove(&position);
        Ok(())
    }

    /// Get a bookmark by ID
    pub fn get_bookmark(&self, bookmark_id: &str) -> Option<&Bookmark> {
        self.bookmarks.values().find(|b| b.id == bookmark_id)
    }

    /// Get a mutable bookmark by ID
    pub fn get_bookmark_mut(&mut self, bookmark_id: &str) -> Option<&mut Bookmark> {
        self.bookmarks.values_mut().find(|b| b.id == bookmark_id)
    }

    /// Get all bookmarks
    pub fn get_all_bookmarks(&self) -> Vec<&Bookmark> {
        self.bookmarks.values().collect()
    }

    /// Get bookmarks of a specific type
    pub fn get_bookmarks_by_type(&self, bookmark_type: BookmarkType) -> Vec<&Bookmark> {
        self.bookmarks
            .values()
            .filter(|b| b.bookmark_type == bookmark_type)
            .collect()
    }

    /// Get the next bookmark after a position
    pub fn get_next_bookmark(&self, position: Duration) -> Option<&Bookmark> {
        self.bookmarks
            .range((
                std::ops::Bound::Excluded(position),
                std::ops::Bound::Unbounded,
            ))
            .next()
            .map(|(_, bookmark)| bookmark)
    }

    /// Get the previous bookmark before a position
    pub fn get_previous_bookmark(&self, position: Duration) -> Option<&Bookmark> {
        self.bookmarks
            .range(..position)
            .next_back()
            .map(|(_, bookmark)| bookmark)
    }

    /// Get the nearest bookmark to a position
    pub fn get_nearest_bookmark(&self, position: Duration) -> Option<&Bookmark> {
        let next = self.get_next_bookmark(position);
        let prev = self.get_previous_bookmark(position);

        match (prev, next) {
            (None, None) => None,
            (Some(p), None) => Some(p),
            (None, Some(n)) => Some(n),
            (Some(p), Some(n)) => {
                let prev_dist = position.saturating_sub(p.position);
                let next_dist = n.position.saturating_sub(position);
                if prev_dist <= next_dist {
                    Some(p)
                } else {
                    Some(n)
                }
            }
        }
    }

    /// Check if auto-bookmark should be created
    pub fn should_create_auto_bookmark(&mut self, position: Duration) -> bool {
        if !self.auto_bookmark_enabled {
            return false;
        }

        match self.last_auto_position {
            None => {
                self.last_auto_position = Some(position);
                true
            }
            Some(last_pos) => {
                let elapsed = position.saturating_sub(last_pos);
                if elapsed >= self.auto_bookmark_interval {
                    self.last_auto_position = Some(position);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Create an auto-bookmark at position
    pub fn create_auto_bookmark(&mut self, position: Duration) -> Result<String, String> {
        let bookmark = Bookmark::new(position, BookmarkType::Auto).with_title("Auto-bookmark");
        self.add_bookmark(bookmark)
    }

    /// Clean up old auto-bookmarks to maintain the limit
    fn cleanup_old_auto_bookmarks(&mut self) {
        let auto_bookmarks: Vec<_> = self
            .bookmarks
            .iter()
            .filter(|(_, b)| b.bookmark_type == BookmarkType::Auto)
            .map(|(pos, b)| (*pos, b.created_at))
            .collect();

        if auto_bookmarks.len() >= self.max_auto_bookmarks {
            // Sort by creation time and remove oldest
            let mut sorted = auto_bookmarks;
            sorted.sort_by_key(|(_, created)| *created);

            let to_remove = sorted.len() - self.max_auto_bookmarks + 1;
            for (pos, _) in sorted.iter().take(to_remove) {
                self.bookmarks.remove(pos);
            }
        }
    }

    /// Clear all bookmarks
    pub fn clear(&mut self) {
        self.bookmarks.clear();
        self.last_auto_position = None;
    }

    /// Clear bookmarks of a specific type
    pub fn clear_by_type(&mut self, bookmark_type: BookmarkType) {
        self.bookmarks
            .retain(|_, b| b.bookmark_type != bookmark_type);
    }

    /// Get bookmark count
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Check if there are any bookmarks
    pub fn is_empty(&self) -> bool {
        self.bookmarks.is_empty()
    }

    /// Export bookmarks to JSON
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.get_all_bookmarks())
            .map_err(|e| format!("Failed to export bookmarks: {}", e))
    }

    /// Import bookmarks from JSON
    pub fn import_json(&mut self, json: &str) -> Result<usize, String> {
        let bookmarks: Vec<Bookmark> =
            serde_json::from_str(json).map_err(|e| format!("Failed to parse bookmarks: {}", e))?;

        let count = bookmarks.len();
        for bookmark in bookmarks {
            self.bookmarks.insert(bookmark.position, bookmark);
        }

        Ok(count)
    }

    /// Set auto-bookmark settings
    pub fn configure_auto_bookmarks(
        &mut self,
        enabled: bool,
        interval_secs: u64,
        max_count: usize,
    ) {
        self.auto_bookmark_enabled = enabled;
        self.auto_bookmark_interval = Duration::from_secs(interval_secs);
        self.max_auto_bookmarks = max_count;
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_creation() {
        let bookmark = Bookmark::new(Duration::from_secs(120), BookmarkType::User)
            .with_title("Chapter 3 Start")
            .with_note("Great quote here");

        assert_eq!(bookmark.position, Duration::from_secs(120));
        assert_eq!(bookmark.bookmark_type, BookmarkType::User);
        assert_eq!(bookmark.title, Some("Chapter 3 Start".to_string()));
        assert_eq!(bookmark.note, Some("Great quote here".to_string()));
        assert_eq!(bookmark.access_count, 0);
    }

    #[test]
    fn test_bookmark_access() {
        let mut bookmark = Bookmark::new(Duration::from_secs(60), BookmarkType::User);
        assert_eq!(bookmark.access_count, 0);
        assert!(bookmark.last_accessed.is_none());

        bookmark.mark_accessed();
        assert_eq!(bookmark.access_count, 1);
        assert!(bookmark.last_accessed.is_some());

        bookmark.mark_accessed();
        assert_eq!(bookmark.access_count, 2);
    }

    #[test]
    fn test_bookmark_display() {
        let bookmark = Bookmark::new(Duration::from_secs(3665), BookmarkType::User)
            .with_title("Important Part");

        let display = bookmark.display_string(Duration::from_secs(7200));
        assert!(display.contains("01:01:05"));
        assert!(display.contains("Important Part"));
        assert!(display.contains("50%"));
    }

    #[test]
    fn test_bookmark_manager_add_remove() {
        let mut manager = BookmarkManager::new();
        manager.set_duration(Duration::from_secs(3600));

        let bookmark = Bookmark::new(Duration::from_secs(120), BookmarkType::User);
        let id = manager.add_bookmark(bookmark.clone()).unwrap();

        assert_eq!(manager.count(), 1);
        assert!(manager.get_bookmark(&id).is_some());

        manager.remove_bookmark(&id).unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_bookmark_manager_navigation() {
        let mut manager = BookmarkManager::new();

        manager
            .add_bookmark(Bookmark::new(Duration::from_secs(100), BookmarkType::User))
            .unwrap();
        manager
            .add_bookmark(Bookmark::new(Duration::from_secs(200), BookmarkType::User))
            .unwrap();
        manager
            .add_bookmark(Bookmark::new(Duration::from_secs(300), BookmarkType::User))
            .unwrap();

        let next = manager.get_next_bookmark(Duration::from_secs(150));
        assert!(next.is_some());
        assert_eq!(next.unwrap().position, Duration::from_secs(200));

        let prev = manager.get_previous_bookmark(Duration::from_secs(250));
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().position, Duration::from_secs(200));

        let nearest = manager.get_nearest_bookmark(Duration::from_secs(180));
        assert!(nearest.is_some());
        assert_eq!(nearest.unwrap().position, Duration::from_secs(200));
    }

    #[test]
    fn test_auto_bookmarks() {
        let mut manager = BookmarkManager::new();
        manager.configure_auto_bookmarks(true, 60, 5); // Every minute, max 5

        assert!(manager.should_create_auto_bookmark(Duration::from_secs(0)));
        assert!(!manager.should_create_auto_bookmark(Duration::from_secs(30)));
        assert!(manager.should_create_auto_bookmark(Duration::from_secs(61)));
        assert!(manager.should_create_auto_bookmark(Duration::from_secs(125)));
    }

    #[test]
    fn test_auto_bookmark_limit() {
        let mut manager = BookmarkManager::new();
        manager.configure_auto_bookmarks(true, 60, 3); // Max 3 auto-bookmarks

        for i in 0..5 {
            manager
                .create_auto_bookmark(Duration::from_secs(i * 60))
                .unwrap();
        }

        let auto_bookmarks = manager.get_bookmarks_by_type(BookmarkType::Auto);
        assert_eq!(auto_bookmarks.len(), 3); // Should only keep 3
    }

    #[test]
    fn test_bookmark_types() {
        let mut manager = BookmarkManager::new();

        manager
            .add_bookmark(Bookmark::new(Duration::from_secs(100), BookmarkType::User))
            .unwrap();
        manager
            .add_bookmark(Bookmark::new(Duration::from_secs(200), BookmarkType::Auto))
            .unwrap();
        manager
            .add_bookmark(Bookmark::new(
                Duration::from_secs(300),
                BookmarkType::Chapter,
            ))
            .unwrap();
        manager
            .add_bookmark(Bookmark::new(
                Duration::from_secs(400),
                BookmarkType::Favorite,
            ))
            .unwrap();
        manager
            .add_bookmark(Bookmark::new(Duration::from_secs(500), BookmarkType::User))
            .unwrap();

        assert_eq!(manager.get_bookmarks_by_type(BookmarkType::User).len(), 2);
        assert_eq!(manager.get_bookmarks_by_type(BookmarkType::Auto).len(), 1);
        assert_eq!(
            manager.get_bookmarks_by_type(BookmarkType::Chapter).len(),
            1
        );
        assert_eq!(
            manager.get_bookmarks_by_type(BookmarkType::Favorite).len(),
            1
        );
    }

    #[test]
    fn test_clear_bookmarks() {
        let mut manager = BookmarkManager::new();

        manager
            .add_bookmark(Bookmark::new(Duration::from_secs(100), BookmarkType::User))
            .unwrap();
        manager
            .add_bookmark(Bookmark::new(Duration::from_secs(200), BookmarkType::Auto))
            .unwrap();
        manager
            .add_bookmark(Bookmark::new(
                Duration::from_secs(300),
                BookmarkType::Favorite,
            ))
            .unwrap();

        manager.clear_by_type(BookmarkType::Auto);
        assert_eq!(manager.count(), 2);

        manager.clear();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_export_import() {
        let mut manager = BookmarkManager::new();

        manager
            .add_bookmark(
                Bookmark::new(Duration::from_secs(100), BookmarkType::User)
                    .with_title("Test Bookmark"),
            )
            .unwrap();

        let json = manager.export_json().unwrap();
        assert!(json.contains("Test Bookmark"));

        let mut new_manager = BookmarkManager::new();
        let count = new_manager.import_json(&json).unwrap();
        assert_eq!(count, 1);
        assert_eq!(new_manager.count(), 1);
    }

    #[test]
    fn test_bookmark_validation() {
        let mut manager = BookmarkManager::new();
        manager.set_duration(Duration::from_secs(100));

        let valid = Bookmark::new(Duration::from_secs(50), BookmarkType::User);
        assert!(manager.add_bookmark(valid).is_ok());

        let invalid = Bookmark::new(Duration::from_secs(200), BookmarkType::User);
        assert!(manager.add_bookmark(invalid).is_err());
    }

    #[test]
    fn test_bookmark_id_generation() {
        let b1 = Bookmark::new(Duration::from_secs(0), BookmarkType::User);
        let b2 = Bookmark::new(Duration::from_secs(0), BookmarkType::User);
        assert_ne!(b1.id, b2.id); // IDs should be unique
    }
}
