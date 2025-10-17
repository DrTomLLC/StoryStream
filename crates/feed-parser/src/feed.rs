// crates/feed-parser/src/feed.rs
//! Feed data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type of feed format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedType {
    /// RSS 2.0 feed
    Rss,
    /// Atom feed
    Atom,
    /// Unknown or unsupported format
    Unknown,
}

/// A parsed feed with metadata and items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    /// Type of feed
    pub feed_type: FeedType,
    /// Feed title
    pub title: String,
    /// Feed description
    pub description: Option<String>,
    /// Feed URL
    pub url: Option<String>,
    /// Feed author
    pub author: Option<String>,
    /// Feed language
    pub language: Option<String>,
    /// Last update time
    pub updated: Option<DateTime<Utc>>,
    /// Feed items/episodes
    pub items: Vec<FeedItem>,
}

impl Feed {
    /// Creates a new feed
    pub fn new(feed_type: FeedType, title: String) -> Self {
        Self {
            feed_type,
            title,
            description: None,
            url: None,
            author: None,
            language: None,
            updated: None,
            items: Vec::new(),
        }
    }

    /// Returns the number of items in the feed
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the feed has no items
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Adds an item to the feed
    pub fn add_item(&mut self, item: FeedItem) {
        self.items.push(item);
    }

    /// Sorts items by publication date (newest first)
    pub fn sort_by_date(&mut self) {
        self.items.sort_by(|a, b| {
            match (&b.published, &a.published) {
                (Some(b_date), Some(a_date)) => b_date.cmp(a_date),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
    }
}

/// A single item in a feed (episode, article, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedItem {
    /// Item title
    pub title: String,
    /// Item description/summary
    pub description: Option<String>,
    /// Item URL/link
    pub url: Option<String>,
    /// Item author
    pub author: Option<String>,
    /// Publication date
    pub published: Option<DateTime<Utc>>,
    /// Media enclosure (audio file)
    pub enclosure: Option<Enclosure>,
    /// Unique identifier
    pub guid: Option<String>,
    /// Duration in seconds
    pub duration: Option<u64>,
}

impl FeedItem {
    /// Creates a new feed item
    pub fn new(title: String) -> Self {
        Self {
            title,
            description: None,
            url: None,
            author: None,
            published: None,
            enclosure: None,
            guid: None,
            duration: None,
        }
    }

    /// Returns true if the item has an audio enclosure
    pub fn has_audio(&self) -> bool {
        self.enclosure
            .as_ref()
            .map(|e| e.is_audio())
            .unwrap_or(false)
    }
}

/// Media enclosure (typically an audio file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enclosure {
    /// URL of the media file
    pub url: String,
    /// MIME type
    pub mime_type: Option<String>,
    /// File size in bytes
    pub length: Option<u64>,
}

impl Enclosure {
    /// Creates a new enclosure
    pub fn new(url: String) -> Self {
        Self {
            url,
            mime_type: None,
            length: None,
        }
    }

    /// Returns true if this is an audio enclosure
    pub fn is_audio(&self) -> bool {
        self.mime_type
            .as_ref()
            .map(|m| m.starts_with("audio/"))
            .unwrap_or(false)
    }

    /// Returns true if this is a video enclosure
    pub fn is_video(&self) -> bool {
        self.mime_type
            .as_ref()
            .map(|m| m.starts_with("video/"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_creation() {
        let feed = Feed::new(FeedType::Rss, "Test Feed".to_string());
        assert_eq!(feed.title, "Test Feed");
        assert_eq!(feed.feed_type, FeedType::Rss);
        assert!(feed.is_empty());
    }

    #[test]
    fn test_feed_add_item() {
        let mut feed = Feed::new(FeedType::Rss, "Test".to_string());
        feed.add_item(FeedItem::new("Item 1".to_string()));
        assert_eq!(feed.item_count(), 1);
        assert!(!feed.is_empty());
    }

    #[test]
    fn test_feed_item_creation() {
        let item = FeedItem::new("Test Item".to_string());
        assert_eq!(item.title, "Test Item");
        assert!(!item.has_audio());
    }

    #[test]
    fn test_feed_item_with_audio() {
        let mut item = FeedItem::new("Test".to_string());
        let mut enclosure = Enclosure::new("http://example.com/audio.mp3".to_string());
        enclosure.mime_type = Some("audio/mpeg".to_string());
        item.enclosure = Some(enclosure);

        assert!(item.has_audio());
    }

    #[test]
    fn test_enclosure_is_audio() {
        let mut enc = Enclosure::new("test.mp3".to_string());
        enc.mime_type = Some("audio/mpeg".to_string());
        assert!(enc.is_audio());
        assert!(!enc.is_video());
    }

    #[test]
    fn test_enclosure_is_video() {
        let mut enc = Enclosure::new("test.mp4".to_string());
        enc.mime_type = Some("video/mp4".to_string());
        assert!(enc.is_video());
        assert!(!enc.is_audio());
    }

    #[test]
    fn test_feed_type_equality() {
        assert_eq!(FeedType::Rss, FeedType::Rss);
        assert_ne!(FeedType::Rss, FeedType::Atom);
    }

    #[test]
    fn test_feed_sort_by_date() {
        let mut feed = Feed::new(FeedType::Rss, "Test".to_string());

        let mut item1 = FeedItem::new("Old".to_string());
        item1.published = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .ok()
            .map(|dt| dt.with_timezone(&Utc));

        let mut item2 = FeedItem::new("New".to_string());
        item2.published = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .ok()
            .map(|dt| dt.with_timezone(&Utc));

        feed.add_item(item1);
        feed.add_item(item2);

        feed.sort_by_date();

        // Newest should be first
        assert_eq!(feed.items[0].title, "New");
        assert_eq!(feed.items[1].title, "Old");
    }
}