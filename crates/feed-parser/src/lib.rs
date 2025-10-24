// crates/feed-parser/src/lib.rs
//! RSS and Atom feed parser for audiobook/podcast feeds
//!
//! This module provides a robust feed parser that can handle:
//! - RSS 2.0 feeds
//! - Atom feeds
//! - Podcast feeds with enclosures
//! - Audiobook feeds
//!
//! # Example
//!
//! ```rust
//! use storystream_feed_parser::FeedParser;
//!
//! let rss = r#"<?xml version="1.0"?>
//! <rss version="2.0">
//!   <channel>
//!     <title>My Podcast</title>
//!     <item>
//!       <title>Episode 1</title>
//!       <enclosure url="http://example.com/ep1.mp3" type="audio/mpeg"/>
//!     </item>
//!   </channel>
//! </rss>"#;
//!
//! let feed = FeedParser::parse(rss).expect("Failed to parse feed");
//! println!("Feed: {} with {} episodes", feed.title, feed.item_count());
//! ```

mod error;
mod feed;
mod parser;

pub use error::{FeedError, FeedResult};
pub use feed::{Enclosure, Feed, FeedItem, FeedType};
pub use parser::FeedParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all types are exported
        let _: FeedType = FeedType::Rss;
        let _: Feed = Feed::new(FeedType::Rss, "Test".to_string());
        let _: FeedItem = FeedItem::new("Test".to_string());
        let _: Enclosure = Enclosure::new("http://example.com".to_string());
    }

    #[test]
    fn test_complete_workflow() {
        let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test Audiobook Feed</title>
    <description>Audiobooks for testing</description>
    <item>
      <title>Book 1 - Chapter 1</title>
      <description>First chapter</description>
      <enclosure url="http://example.com/book1_ch1.mp3" type="audio/mpeg" length="5000000"/>
      <pubDate>Mon, 01 Jan 2024 12:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>"#;

        let feed = FeedParser::parse(rss).expect("Should parse");
        assert_eq!(feed.title, "Test Audiobook Feed");
        assert_eq!(feed.item_count(), 1);

        let item = &feed.items[0];
        assert_eq!(item.title, "Book 1 - Chapter 1");
        assert!(item.has_audio());
        assert!(item.published.is_some());
    }
}
