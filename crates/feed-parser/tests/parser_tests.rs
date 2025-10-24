// crates/feed-parser/tests/parser_tests.rs
//! Comprehensive feed parser tests

use storystream_feed_parser::{FeedParser, FeedType};

#[test]
fn test_parse_complex_rss() {
    let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
  <channel>
    <title>The Great Audiobook Podcast</title>
    <description>Classic literature read aloud</description>
    <link>http://example.com/podcast</link>
    <language>en-us</language>
    <item>
      <title>Pride and Prejudice - Chapter 1</title>
      <description>It is a truth universally acknowledged...</description>
      <link>http://example.com/pride1</link>
      <guid>http://example.com/pride1</guid>
      <pubDate>Mon, 01 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="http://example.com/pride1.mp3" type="audio/mpeg" length="10000000"/>
      <author>Jane Austen</author>
    </item>
    <item>
      <title>Pride and Prejudice - Chapter 2</title>
      <description>Mr. Bennet was among the earliest...</description>
      <pubDate>Tue, 02 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="http://example.com/pride2.mp3" type="audio/mpeg" length="9500000"/>
    </item>
  </channel>
</rss>"#;

    let feed = FeedParser::parse(rss).expect("Should parse complex RSS");

    assert_eq!(feed.feed_type, FeedType::Rss);
    assert_eq!(feed.title, "The Great Audiobook Podcast");
    assert_eq!(
        feed.description,
        Some("Classic literature read aloud".to_string())
    );
    assert_eq!(feed.url, Some("http://example.com/podcast".to_string()));
    assert_eq!(feed.language, Some("en-us".to_string()));
    assert_eq!(feed.item_count(), 2);

    // Check first item
    let item1 = &feed.items[0];
    assert_eq!(item1.title, "Pride and Prejudice - Chapter 1");
    assert!(item1.description.is_some());
    assert_eq!(item1.url, Some("http://example.com/pride1".to_string()));
    assert_eq!(item1.author, Some("Jane Austen".to_string()));
    assert!(item1.published.is_some());
    assert!(item1.has_audio());

    // Check enclosure
    let enclosure = item1.enclosure.as_ref().expect("Should have enclosure");
    assert_eq!(enclosure.url, "http://example.com/pride1.mp3");
    assert_eq!(enclosure.mime_type, Some("audio/mpeg".to_string()));
    assert_eq!(enclosure.length, Some(10000000));
}

#[test]
fn test_parse_atom_feed() {
    let atom = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Atom Audiobook Feed</title>
  <subtitle>Books in Atom format</subtitle>
  <link href="http://example.com/atom"/>
  <entry>
    <title>Moby Dick - Part 1</title>
    <link href="http://example.com/moby1"/>
    <id>urn:uuid:1234</id>
    <published>2024-01-01T12:00:00Z</published>
    <summary>Call me Ishmael...</summary>
  </entry>
  <entry>
    <title>Moby Dick - Part 2</title>
    <id>urn:uuid:1235</id>
    <updated>2024-01-02T12:00:00Z</updated>
  </entry>
</feed>"#;

    let feed = FeedParser::parse(atom).expect("Should parse Atom");

    assert_eq!(feed.feed_type, FeedType::Atom);
    assert_eq!(feed.title, "Atom Audiobook Feed");
    assert_eq!(feed.description, Some("Books in Atom format".to_string()));
    assert_eq!(feed.item_count(), 2);

    let item1 = &feed.items[0];
    assert_eq!(item1.title, "Moby Dick - Part 1");
    assert_eq!(item1.url, Some("http://example.com/moby1".to_string()));
    assert!(item1.published.is_some());
}

#[test]
fn test_feed_sorting() {
    // Use a simpler date format that's guaranteed to work
    let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test</title>
    <item>
      <title>Item A</title>
      <pubDate>Mon, 01 Jan 2024 00:00:00 +0000</pubDate>
    </item>
    <item>
      <title>Item B</title>
      <pubDate>Tue, 02 Jan 2024 00:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>"#;

    let mut feed = FeedParser::parse(rss).expect("Should parse");

    // Before sorting, Item A should be first
    assert_eq!(feed.items[0].title, "Item A");
    assert_eq!(feed.items[1].title, "Item B");

    feed.sort_by_date();

    // After sorting (newest first), Item B should be first
    assert_eq!(feed.items[0].title, "Item B");
    assert_eq!(feed.items[1].title, "Item A");
}

#[test]
fn test_multiple_audio_formats() {
    let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Multi-Format Feed</title>
    <item>
      <title>MP3 Episode</title>
      <enclosure url="http://example.com/ep1.mp3" type="audio/mpeg"/>
    </item>
    <item>
      <title>OGG Episode</title>
      <enclosure url="http://example.com/ep2.ogg" type="audio/ogg"/>
    </item>
    <item>
      <title>M4A Episode</title>
      <enclosure url="http://example.com/ep3.m4a" type="audio/mp4"/>
    </item>
    <item>
      <title>Video Episode</title>
      <enclosure url="http://example.com/ep4.mp4" type="video/mp4"/>
    </item>
  </channel>
</rss>"#;

    let feed = FeedParser::parse(rss).expect("Should parse");

    assert!(feed.items[0].has_audio());
    assert!(feed.items[1].has_audio());
    assert!(feed.items[2].has_audio());
    assert!(!feed.items[3].has_audio()); // Video, not audio
}

#[test]
fn test_empty_feed() {
    let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Empty Feed</title>
    <description>No items</description>
  </channel>
</rss>"#;

    let feed = FeedParser::parse(rss).expect("Should parse empty feed");
    assert!(feed.is_empty());
    assert_eq!(feed.item_count(), 0);
}

#[test]
fn test_malformed_xml() {
    let bad_xml = "<rss><channel><title>Unclosed";
    let result = FeedParser::parse(bad_xml);
    assert!(result.is_err());
}

#[test]
fn test_unknown_format() {
    let html = "<html><body>Not a feed</body></html>";
    let result = FeedParser::parse(html);
    assert!(result.is_err());
}

#[test]
fn test_feed_with_special_characters() {
    let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Feed with &amp; Special &lt;Characters&gt;</title>
    <item>
      <title>Episode with "Quotes" &amp; Symbols</title>
      <description>Testing &lt;html&gt; entities</description>
    </item>
  </channel>
</rss>"#;

    let feed = FeedParser::parse(rss).expect("Should handle special chars");
    assert!(feed.title.contains("&"));
    assert!(feed.title.contains("<"));
}
