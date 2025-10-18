// crates/feed-parser/tests/integration_tests.rs
//! Integration tests for feed parser

use storystream_feed_parser::{FeedParser, FeedType};

#[test]
fn test_parse_complex_rss_feed() {
    let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="https://www.itunes.com/dtds/podcast-1.0.dtd">
  <channel>
    <title>The Great Audiobook Podcast</title>
    <description>Classic literature read aloud</description>
    <link>https://example.com/podcast</link>
    <language>en-us</language>

    <item>
      <title>Pride and Prejudice - Chapter 1</title>
      <description>It is a truth universally acknowledged...</description>
      <link>https://example.com/pride1</link>
      <guid>https://example.com/pride1</guid>
      <pubDate>Mon, 01 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="https://example.com/pride1.mp3" type="audio/mpeg" length="10000000"/>
      <author>Jane Austen</author>
    </item>

    <item>
      <title>Pride and Prejudice - Chapter 2</title>
      <description>Mr. Bennet was among the earliest...</description>
      <pubDate>Tue, 02 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="https://example.com/pride2.mp3" type="audio/mpeg" length="9500000"/>
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
    assert_eq!(feed.url, Some("https://example.com/podcast".to_string()));
    assert_eq!(feed.language, Some("en-us".to_string()));
    assert_eq!(feed.item_count(), 2);

    // Check first item
    let item1 = &feed.items[0];
    assert_eq!(item1.title, "Pride and Prejudice - Chapter 1");
    assert!(item1.description.is_some());
    assert_eq!(item1.url, Some("https://example.com/pride1".to_string()));
    assert_eq!(item1.author, Some("Jane Austen".to_string()));
    assert!(item1.published.is_some());
    assert!(item1.has_audio());

    // Check enclosure
    let enclosure = item1.enclosure.as_ref().expect("Should have enclosure");
    assert_eq!(enclosure.url, "https://example.com/pride1.mp3");
    assert_eq!(enclosure.mime_type, Some("audio/mpeg".to_string()));
    assert_eq!(enclosure.length, Some(10000000));
}

#[test]
fn test_parse_atom_feed() {
    let atom = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="https://www.w3.org/2005/Atom">
  <title>Atom Audiobook Feed</title>
  <subtitle>Books in Atom format</subtitle>
  <link href="https://example.com/atom"/>

  <entry>
    <title>Moby Dick - Part 1</title>
    <link href="https://example.com/moby1"/>
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
    assert_eq!(
        feed.description,
        Some("Books in Atom format".to_string())
    );
    assert_eq!(feed.item_count(), 2);

    let item1 = &feed.items[0];
    assert_eq!(item1.title, "Moby Dick - Part 1");
    assert_eq!(item1.url, Some("https://example.com/moby1".to_string()));
    assert!(item1.published.is_some());
}

#[test]
fn test_feed_sorting() {
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

    // Before sorting
    assert_eq!(feed.items[0].title, "Item A");
    assert_eq!(feed.items[1].title, "Item B");

    feed.sort_by_date();

    // After sorting (newest first)
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
      <enclosure url="https://example.com/ep1.mp3" type="audio/mpeg"/>
    </item>
    <item>
      <title>OGG Episode</title>
      <enclosure url="https://example.com/ep2.ogg" type="audio/ogg"/>
    </item>
    <item>
      <title>M4A Episode</title>
      <enclosure url="https://example.com/ep3.m4a" type="audio/mp4"/>
    </item>
    <item>
      <title>Video Episode</title>
      <enclosure url="https://example.com/ep4.mp4" type="video/mp4"/>
    </item>
  </channel>
</rss>"#;

    let feed = FeedParser::parse(rss).expect("Should parse");

    assert!(feed.items[0].has_audio());
    assert!(feed.items[1].has_audio());
    assert!(feed.items[2].has_audio());
    assert!(!feed.items[3].has_audio()); // Video, not audio

    let audio_items = feed.audio_items();
    assert_eq!(audio_items.len(), 3);
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

    // Debug output to help diagnose the issue
    eprintln!("Feed title: {:?}", feed.title);
    eprintln!("Feed title bytes: {:?}", feed.title.as_bytes());

    // XML entities should be decoded:
    // &amp; → &
    // &lt; → <
    // &gt; → >
    assert!(
        feed.title.contains('&'),
        "Feed title should contain '&' character. Got: {:?}",
        feed.title
    );
    assert!(
        feed.title.contains('<'),
        "Feed title should contain '<' character. Got: {:?}",
        feed.title
    );
    assert!(
        feed.title.contains('>'),
        "Feed title should contain '>' character. Got: {:?}",
        feed.title
    );

    // Verify the full expected string
    assert_eq!(
        feed.title,
        "Feed with & Special <Characters>",
        "Feed title should have all XML entities decoded"
    );

    // Check item title too
    assert!(feed.items[0].title.contains('&'));
    assert_eq!(
        feed.items[0].title,
        "Episode with \"Quotes\" & Symbols"
    );
}

#[test]
fn test_rss_without_enclosures() {
    let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Text Only Feed</title>
    <item>
      <title>Article 1</title>
      <description>Just text content</description>
    </item>
  </channel>
</rss>"#;

    let feed = FeedParser::parse(rss).expect("Should parse");
    assert_eq!(feed.item_count(), 1);
    assert!(!feed.items[0].has_audio());
    assert!(feed.audio_items().is_empty());
}

#[test]
fn test_feed_with_missing_optional_fields() {
    let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Minimal Feed</title>
    <item>
      <title>Minimal Item</title>
    </item>
  </channel>
</rss>"#;

    let feed = FeedParser::parse(rss).expect("Should parse minimal feed");
    let item = &feed.items[0];

    assert_eq!(item.title, "Minimal Item");
    assert!(item.description.is_none());
    assert!(item.url.is_none());
    assert!(item.author.is_none());
    assert!(item.published.is_none());
}

#[test]
fn test_date_parsing_rfc2822() {
    let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Date Test</title>
    <item>
      <title>Item</title>
      <pubDate>Wed, 15 Jan 2025 14:30:00 +0000</pubDate>
    </item>
  </channel>
</rss>"#;

    let feed = FeedParser::parse(rss).expect("Should parse");
    assert!(feed.items[0].published.is_some());
}

#[test]
fn test_atom_date_parsing_rfc3339() {
    let atom = r#"<?xml version="1.0"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Date Test</title>
  <entry>
    <title>Item</title>
    <id>123</id>
    <published>2025-01-15T14:30:00Z</published>
  </entry>
</feed>"#;

    let feed = FeedParser::parse(atom).expect("Should parse");
    assert!(feed.items[0].published.is_some());
}

#[test]
fn test_large_feed_performance() {
    // Generate feed with 1000 items
    let mut rss = String::from(
        r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Large Feed</title>"#,
    );

    for i in 0..1000 {
        rss.push_str(&format!(
            r#"
    <item>
      <title>Episode {}</title>
      <enclosure url="https://example.com/ep{}.mp3" type="audio/mpeg"/>
    </item>"#,
            i, i
        ));
    }

    rss.push_str(
        r#"
  </channel>
</rss>"#,
    );

    let start = std::time::Instant::now();
    let feed = FeedParser::parse(&rss).expect("Should parse large feed");
    let duration = start.elapsed();

    assert_eq!(feed.item_count(), 1000);
    assert!(duration.as_millis() < 100, "Parsing took too long: {:?}", duration);
}