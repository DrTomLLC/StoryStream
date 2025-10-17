// crates/feed-parser/src/parser.rs
//! Feed parsing logic

use crate::error::{FeedError, FeedResult};
use crate::feed::{Enclosure, Feed, FeedItem, FeedType};
use chrono::DateTime;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Feed parser
pub struct FeedParser;

impl FeedParser {
    /// Parses a feed from a string
    pub fn parse(content: &str) -> FeedResult<Feed> {
        let feed_type = Self::detect_type(content)?;

        match feed_type {
            FeedType::Rss => Self::parse_rss(content),
            FeedType::Atom => Self::parse_atom(content),
            FeedType::Unknown => Err(FeedError::UnsupportedFormat("Unknown feed format".to_string())),
        }
    }

    /// Detects the feed type from content
    fn detect_type(content: &str) -> FeedResult<FeedType> {
        if content.contains("<rss") {
            Ok(FeedType::Rss)
        } else if content.contains("<feed") && content.contains("xmlns=\"http://www.w3.org/2005/Atom\"") {
            Ok(FeedType::Atom)
        } else {
            Ok(FeedType::Unknown)
        }
    }

    /// Parses an RSS feed
    fn parse_rss(content: &str) -> FeedResult<Feed> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut feed = Feed::new(FeedType::Rss, String::new());
        let mut current_item: Option<FeedItem> = None;
        let mut text_buffer = String::new();
        let mut in_item = false;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if element_name == "item" {
                        in_item = true;
                        current_item = Some(FeedItem::new(String::new()));
                    }
                }
                Ok(Event::Empty(e)) => {
                    // Handle self-closing tags like <enclosure ... />
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if element_name == "enclosure" && in_item {
                        let mut url = None;
                        let mut mime_type = None;
                        let mut length = None;

                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                let value = String::from_utf8_lossy(&attr.value);

                                match key.as_ref() {
                                    "url" => url = Some(value.to_string()),
                                    "type" => mime_type = Some(value.to_string()),
                                    "length" => length = value.parse().ok(),
                                    _ => {}
                                }
                            }
                        }

                        if let Some(url) = url {
                            if let Some(item) = current_item.as_mut() {
                                item.enclosure = Some(Enclosure {
                                    url,
                                    mime_type,
                                    length,
                                });
                            }
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    text_buffer = e.unescape().map(|s| s.to_string()).unwrap_or_default();
                }
                Ok(Event::End(e)) => {
                    // Convert to owned string to avoid lifetime issues
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if in_item {
                        if let Some(item) = current_item.as_mut() {
                            match element_name.as_str() {
                                "title" => item.title = text_buffer.clone(),
                                "description" => item.description = Some(text_buffer.clone()),
                                "link" => item.url = Some(text_buffer.clone()),
                                "author" => item.author = Some(text_buffer.clone()),
                                "pubDate" => {
                                    item.published = DateTime::parse_from_rfc2822(&text_buffer)
                                        .ok()
                                        .map(|dt| dt.with_timezone(&chrono::Utc));
                                }
                                "guid" => item.guid = Some(text_buffer.clone()),
                                _ => {}
                            }
                        }

                        if element_name == "item" {
                            if let Some(item) = current_item.take() {
                                feed.add_item(item);
                            }
                            in_item = false;
                        }
                    } else {
                        match element_name.as_str() {
                            "title" if feed.title.is_empty() => feed.title = text_buffer.clone(),
                            "description" => feed.description = Some(text_buffer.clone()),
                            "link" => feed.url = Some(text_buffer.clone()),
                            "language" => feed.language = Some(text_buffer.clone()),
                            _ => {}
                        }
                    }

                    text_buffer.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(FeedError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        if feed.title.is_empty() {
            return Err(FeedError::MissingField("title".to_string()));
        }

        Ok(feed)
    }

    /// Parses an Atom feed
    fn parse_atom(content: &str) -> FeedResult<Feed> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut feed = Feed::new(FeedType::Atom, String::new());
        let mut current_item: Option<FeedItem> = None;
        let mut text_buffer = String::new();
        let mut in_entry = false;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if element_name == "entry" {
                        in_entry = true;
                        current_item = Some(FeedItem::new(String::new()));
                    }

                    // Handle link elements (can be self-closing)
                    if element_name == "link" {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                if key == "href" {
                                    let href = String::from_utf8_lossy(&attr.value).to_string();
                                    if in_entry {
                                        if let Some(item) = current_item.as_mut() {
                                            item.url = Some(href);
                                        }
                                    } else {
                                        feed.url = Some(href);
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    text_buffer = e.unescape().map(|s| s.to_string()).unwrap_or_default();
                }
                Ok(Event::End(e)) => {
                    // Convert to owned string to avoid lifetime issues
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if in_entry {
                        if let Some(item) = current_item.as_mut() {
                            match element_name.as_str() {
                                "title" => item.title = text_buffer.clone(),
                                "summary" | "content" => item.description = Some(text_buffer.clone()),
                                "author" => item.author = Some(text_buffer.clone()),
                                "published" | "updated" => {
                                    item.published = DateTime::parse_from_rfc3339(&text_buffer)
                                        .ok()
                                        .map(|dt| dt.with_timezone(&chrono::Utc));
                                }
                                "id" => item.guid = Some(text_buffer.clone()),
                                _ => {}
                            }
                        }

                        if element_name == "entry" {
                            if let Some(item) = current_item.take() {
                                feed.add_item(item);
                            }
                            in_entry = false;
                        }
                    } else {
                        match element_name.as_str() {
                            "title" if feed.title.is_empty() => feed.title = text_buffer.clone(),
                            "subtitle" => feed.description = Some(text_buffer.clone()),
                            _ => {}
                        }
                    }

                    text_buffer.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(FeedError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        if feed.title.is_empty() {
            return Err(FeedError::MissingField("title".to_string()));
        }

        Ok(feed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rss() {
        let rss = r#"<?xml version="1.0"?><rss version="2.0"><channel></channel></rss>"#;
        let feed_type = FeedParser::detect_type(rss).expect("Should detect type");
        assert_eq!(feed_type, FeedType::Rss);
    }

    #[test]
    fn test_detect_atom() {
        let atom = r#"<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom"></feed>"#;
        let feed_type = FeedParser::detect_type(atom).expect("Should detect type");
        assert_eq!(feed_type, FeedType::Atom);
    }

    #[test]
    fn test_parse_minimal_rss() {
        let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <description>A test feed</description>
    <link>http://example.com</link>
  </channel>
</rss>"#;

        let feed = FeedParser::parse(rss).expect("Should parse RSS");
        assert_eq!(feed.title, "Test Feed");
        assert_eq!(feed.description, Some("A test feed".to_string()));
        assert!(feed.is_empty());
    }

    #[test]
    fn test_parse_rss_with_items() {
        let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <item>
      <title>Episode 1</title>
      <description>First episode</description>
      <enclosure url="http://example.com/ep1.mp3" type="audio/mpeg" length="1000"/>
    </item>
    <item>
      <title>Episode 2</title>
    </item>
  </channel>
</rss>"#;

        let feed = FeedParser::parse(rss).expect("Should parse RSS");
        assert_eq!(feed.item_count(), 2);
        assert_eq!(feed.items[0].title, "Episode 1");
        assert!(feed.items[0].has_audio());
    }

    #[test]
    fn test_parse_invalid_xml() {
        let invalid = "not xml at all";
        let result = FeedParser::parse(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rss_missing_title() {
        let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <description>No title</description>
  </channel>
</rss>"#;

        let result = FeedParser::parse(rss);
        assert!(result.is_err());
    }
}