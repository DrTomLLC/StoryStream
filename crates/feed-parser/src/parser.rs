// crates/feed-parser/src/parser.rs
//! Feed parsing logic - ZERO PANICS - quick-xml 0.36 compatible
//! SECURITY: Uses HTTPS URLs in tests and documentation
//! FIXED: HTML entity unescaping working correctly with quick-xml 0.36
//! FIXED: Atom https namespace support

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
            FeedType::Unknown => Err(FeedError::UnsupportedFormat(
                "Unknown feed format".to_string(),
            )),
        }
    }

    /// Detects the feed type from content
    fn detect_type(content: &str) -> FeedResult<FeedType> {
        if content.contains("<rss") {
            Ok(FeedType::Rss)
        } else if content.contains("<feed")
            && (content.contains("xmlns=\"http://www.w3.org/2005/Atom\"")
            || content.contains("xmlns=\"https://www.w3.org/2005/Atom\"")
            || content.contains("http://www.w3.org/2005/Atom")
            || content.contains("https://www.w3.org/2005/Atom"))
        {
            Ok(FeedType::Atom)
        } else {
            Ok(FeedType::Unknown)
        }
    }

    /// Parses an RSS feed
    fn parse_rss(content: &str) -> FeedResult<Feed> {
        let mut reader = Reader::from_str(content);
        // DON'T trim_text - it removes spaces around decoded entities!

        let mut feed = Feed::new(FeedType::Rss, String::new());
        let mut current_item: Option<FeedItem> = None;
        let mut text_buffer = String::new();
        let mut in_item = false;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if element_name == "item" {
                        in_item = true;
                        current_item = Some(FeedItem::new(String::new()));
                    } else if element_name == "enclosure" {
                        // Parse enclosure attributes
                        let mut url = String::new();
                        let mut mime_type = None;
                        let mut length = None;

                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();

                                match key.as_str() {
                                    "url" => url = value,
                                    "type" => mime_type = Some(value),
                                    "length" => length = value.parse().ok(),
                                    _ => {}
                                }
                            }
                        }

                        if !url.is_empty() {
                            if let Some(ref mut item) = current_item {
                                let mut enc = Enclosure::new(url);
                                enc.mime_type = mime_type;
                                enc.length = length;
                                item.enclosure = Some(enc);
                            }
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    // CRITICAL FIX: In quick-xml 0.36, use e.unescape()
                    // Returns Result<Cow<'_, str>, EscapeError>
                    // Cow<str> auto-derefs to &str when passed to push_str()
                    // This decodes HTML entities: &amp; → &, &lt; → <, &gt; → >, etc.
                    if let Ok(unescaped) = e.unescape() {
                        text_buffer.push_str(&unescaped);
                    }
                }
                Ok(Event::End(e)) => {
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if in_item {
                        if let Some(ref mut item) = current_item {
                            // Trim whitespace from accumulated text
                            let trimmed = text_buffer.trim();

                            match element_name.as_str() {
                                "title" if item.title.is_empty() => {
                                    item.title = trimmed.to_string()
                                }
                                "description" => item.description = Some(trimmed.to_string()),
                                "link" => item.url = Some(trimmed.to_string()),
                                "author" => item.author = Some(trimmed.to_string()),
                                "pubDate" => {
                                    item.published = DateTime::parse_from_rfc2822(trimmed)
                                        .ok()
                                        .map(|dt| dt.with_timezone(&chrono::Utc));
                                }
                                "guid" => item.guid = Some(trimmed.to_string()),
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
                        // Trim whitespace from accumulated text
                        let trimmed = text_buffer.trim();

                        match element_name.as_str() {
                            "title" if feed.title.is_empty() => feed.title = trimmed.to_string(),
                            "description" => feed.description = Some(trimmed.to_string()),
                            "link" => feed.url = Some(trimmed.to_string()),
                            "language" => feed.language = Some(trimmed.to_string()),
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
        // DON'T trim_text - it removes spaces around decoded entities!

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
                    } else if element_name == "link" {
                        // Extract href attribute from link element (handles both <link> and <link/>)
                        let mut href = None;
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                if key == "href" {
                                    href = Some(String::from_utf8_lossy(&attr.value).to_string());
                                }
                            }
                        }

                        if let Some(url) = href {
                            if in_entry {
                                if let Some(ref mut item) = current_item {
                                    item.url = Some(url);
                                }
                            } else {
                                feed.url = Some(url);
                            }
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    // CRITICAL FIX: In quick-xml 0.36, use e.unescape()
                    // Returns Result<Cow<'_, str>, EscapeError>
                    // Cow<str> auto-derefs to &str when passed to push_str()
                    // This decodes HTML entities: &amp; → &, &lt; → <, &gt; → >, etc.
                    if let Ok(unescaped) = e.unescape() {
                        text_buffer.push_str(&unescaped);
                    }
                }
                Ok(Event::End(e)) => {
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if in_entry {
                        if let Some(ref mut item) = current_item {
                            // Trim whitespace from accumulated text
                            let trimmed = text_buffer.trim();

                            match element_name.as_str() {
                                "title" if item.title.is_empty() => {
                                    item.title = trimmed.to_string()
                                }
                                "summary" | "content" => item.description = Some(trimmed.to_string()),
                                "author" => item.author = Some(trimmed.to_string()),
                                "published" | "updated" => {
                                    item.published = DateTime::parse_from_rfc3339(trimmed)
                                        .ok()
                                        .map(|dt| dt.with_timezone(&chrono::Utc));
                                }
                                "id" => item.guid = Some(trimmed.to_string()),
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
                        // Trim whitespace from accumulated text
                        let trimmed = text_buffer.trim();

                        match element_name.as_str() {
                            "title" if feed.title.is_empty() => feed.title = trimmed.to_string(),
                            "subtitle" => feed.description = Some(trimmed.to_string()),
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
        match FeedParser::detect_type(rss) {
            Ok(feed_type) => assert_eq!(feed_type, FeedType::Rss),
            Err(_) => panic!("Should detect type"),
        }
    }

    #[test]
    fn test_detect_atom() {
        let atom = r#"<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom"></feed>"#;
        match FeedParser::detect_type(atom) {
            Ok(feed_type) => assert_eq!(feed_type, FeedType::Atom),
            Err(_) => panic!("Should detect type"),
        }
    }

    #[test]
    fn test_detect_atom_https() {
        let atom = r#"<?xml version="1.0"?><feed xmlns="https://www.w3.org/2005/Atom"></feed>"#;
        match FeedParser::detect_type(atom) {
            Ok(feed_type) => assert_eq!(feed_type, FeedType::Atom),
            Err(_) => panic!("Should detect https Atom namespace"),
        }
    }

    #[test]
    fn test_parse_minimal_rss() {
        let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <description>A test feed</description>
    <link>https://example.com</link>
  </channel>
</rss>"#;

        match FeedParser::parse(rss) {
            Ok(feed) => {
                assert_eq!(feed.title, "Test Feed");
                assert_eq!(feed.description, Some("A test feed".to_string()));
                assert!(feed.is_empty());
            }
            Err(e) => panic!("Should parse RSS: {}", e),
        }
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
      <enclosure url="https://example.com/ep1.mp3" type="audio/mpeg" length="1000"/>
    </item>
    <item>
      <title>Episode 2</title>
    </item>
  </channel>
</rss>"#;

        match FeedParser::parse(rss) {
            Ok(feed) => {
                assert_eq!(feed.item_count(), 2);
                assert_eq!(feed.items[0].title, "Episode 1");
                assert!(feed.items[0].has_audio());
            }
            Err(e) => panic!("Should parse RSS: {}", e),
        }
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

    #[test]
    fn test_parse_atom_basic() {
        let atom = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Test Atom Feed</title>
  <subtitle>A test feed</subtitle>
  <entry>
    <title>Entry 1</title>
    <id>entry1</id>
  </entry>
</feed>"#;

        match FeedParser::parse(atom) {
            Ok(feed) => {
                assert_eq!(feed.feed_type, FeedType::Atom);
                assert_eq!(feed.title, "Test Atom Feed");
                assert_eq!(feed.item_count(), 1);
            }
            Err(e) => panic!("Should parse Atom: {}", e),
        }
    }

    #[test]
    fn test_html_entity_unescaping() {
        let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Feed with &amp; Special &lt;Characters&gt;</title>
    <item>
      <title>Episode with "Quotes" &amp; Symbols</title>
    </item>
  </channel>
</rss>"#;

        match FeedParser::parse(rss) {
            Ok(feed) => {
                // Verify HTML entities are properly decoded
                assert!(
                    feed.title.contains('&'),
                    "Feed title should contain ampersand. Got: {:?}",
                    feed.title
                );
                assert!(
                    feed.title.contains('<'),
                    "Feed title should contain less-than. Got: {:?}",
                    feed.title
                );
                assert!(
                    feed.title.contains('>'),
                    "Feed title should contain greater-than. Got: {:?}",
                    feed.title
                );
                assert_eq!(feed.title, "Feed with & Special <Characters>");

                assert!(
                    feed.items[0].title.contains('&'),
                    "Item title should contain ampersand. Got: {:?}",
                    feed.items[0].title
                );
                assert_eq!(feed.items[0].title, "Episode with \"Quotes\" & Symbols");
            }
            Err(e) => panic!("Should parse RSS with entities: {}", e),
        }
    }
}