# StoryStream Feed Parser

**Production-ready RSS and Atom feed parser for audiobooks and podcasts.**

## Features

- ✅ **RSS 2.0 Support** - Full RSS parsing with enclosures
- ✅ **Atom Support** - RFC 4287 compliant Atom feed parsing
- ✅ **Audio Detection** - Automatic identification of audio enclosures
- ✅ **Date Parsing** - RFC 2822 (RSS) and RFC 3339 (Atom) date formats
- ✅ **Zero Panics** - All errors returned via Result types
- ✅ **Thoroughly Tested** - 25+ unit and integration tests
- ✅ **Minimal Dependencies** - Only `quick-xml`, `chrono`, `serde`, `thiserror`

## Usage

```rust
use storystream_feed_parser::FeedParser;

let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Classic Audiobooks</title>
    <item>
      <title>Pride and Prejudice - Chapter 1</title>
      <enclosure url="http://example.com/ch1.mp3" 
                 type="audio/mpeg" 
                 length="15000000"/>
    </item>
  </channel>
</rss>"#;

let feed = FeedParser::parse(rss)?;

println!("Feed: {}", feed.title);
println!("Episodes: {}", feed.item_count());

for item in feed.audio_items() {
    println!("  - {}", item.title);
    if let Some(url) = item.audio_url() {
        println!("    Audio: {}", url);
    }
}
```

## API Reference

### `FeedParser::parse(content: &str) -> FeedResult<Feed>`

Parses RSS or Atom feed from string. Automatically detects feed type.

**Returns:**
- `Ok(Feed)` - Successfully parsed feed
- `Err(FeedError)` - Parse error with details

### `Feed` Structure

```rust
pub struct Feed {
    pub feed_type: FeedType,      // Rss or Atom
    pub title: String,             // Feed title
    pub description: Option<String>,
    pub url: Option<String>,
    pub language: Option<String>,
    pub items: Vec<FeedItem>,      // Feed entries
}
```

**Methods:**
- `item_count() -> usize` - Number of items
- `is_empty() -> bool` - Check if feed has items
- `sort_by_date()` - Sort items by date (newest first)
- `audio_items() -> Vec<&FeedItem>` - Filter to audio-only items

### `FeedItem` Structure

```rust
pub struct FeedItem {
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub enclosure: Option<Enclosure>,
}
```

**Methods:**
- `has_audio() -> bool` - Check if item has audio enclosure
- `audio_url() -> Option<&str>` - Get audio URL if available

### `Enclosure` Structure

```rust
pub struct Enclosure {
    pub url: String,
    pub mime_type: Option<String>,  // e.g., "audio/mpeg"
    pub length: Option<u64>,        // Size in bytes
}
```

**Methods:**
- `is_audio() -> bool` - Check if MIME type is audio/*
- `is_video() -> bool` - Check if MIME type is video/*

## Error Handling

All errors are returned via the `FeedError` enum:

```rust
pub enum FeedError {
    InvalidXml(String),           // Malformed XML
    UnsupportedFormat(String),    // Not RSS or Atom
    MissingField(String),         // Required field missing
    InvalidDate(String),          // Date parse error
    XmlParse(String),             // XML parser error
}
```

**Example:**

```rust
match FeedParser::parse(content) {
    Ok(feed) => println!("Parsed: {}", feed.title),
    Err(FeedError::InvalidXml(msg)) => eprintln!("Bad XML: {}", msg),
    Err(FeedError::MissingField(field)) => eprintln!("Missing: {}", field),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Supported Feed Formats

### RSS 2.0

```xml
<rss version="2.0">
  <channel>
    <title>Feed Title</title>
    <description>Feed description</description>
    <link>http://example.com</link>
    <language>en-us</language>
    
    <item>
      <title>Episode Title</title>
      <description>Episode description</description>
      <link>http://example.com/episode</link>
      <pubDate>Mon, 01 Jan 2024 12:00:00 GMT</pubDate>
      <author>Author Name</author>
      <guid>unique-id</guid>
      <enclosure url="http://example.com/audio.mp3" 
                 type="audio/mpeg" 
                 length="10000000"/>
    </item>
  </channel>
</rss>
```

### Atom 1.0

```xml
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Feed Title</title>
  <subtitle>Feed description</subtitle>
  <link href="http://example.com"/>
  
  <entry>
    <title>Episode Title</title>
    <summary>Episode description</summary>
    <link href="http://example.com/episode"/>
    <id>unique-id</id>
    <published>2024-01-01T12:00:00Z</published>
    <author><name>Author Name</name></author>
  </entry>
</feed>
```

## Examples

### Parse LibriVox Feed

```rust
use storystream_feed_parser::FeedParser;

// Fetch LibriVox RSS feed
let content = reqwest::get("https://librivox.org/rss/latest_releases")
    .await?
    .text()
    .await?;

let feed = FeedParser::parse(&content)?;

println!("Latest LibriVox releases:");
for item in feed.audio_items().iter().take(10) {
    println!("  📖 {}", item.title);
    if let Some(author) = &item.author {
        println!("     by {}", author);
    }
}
```

### Filter by Date

```rust
use chrono::{Duration, Utc};

let feed = FeedParser::parse(rss_content)?;
let one_week_ago = Utc::now() - Duration::days(7);

let recent = feed.items.iter()
    .filter(|item| {
        item.published
            .map_or(false, |date| date > one_week_ago)
    })
    .collect::<Vec<_>>();

println!("Episodes from last week: {}", recent.len());
```

### Download All Audio Files

```rust
use storystream_feed_parser::FeedParser;

let feed = FeedParser::parse(content)?;

for item in feed.audio_items() {
    if let Some(url) = item.audio_url() {
        println!("Downloading: {}", item.title);
        // Use storystream-network crate to download
        // network::download(url, &format!("{}.mp3", item.title)).await?;
    }
}
```

### Sort by Publication Date

```rust
let mut feed = FeedParser::parse(content)?;

// Sort newest first
feed.sort_by_date();

println!("Latest episode: {}", feed.items[0].title);
```

## Testing

Run the test suite:

```bash
# Unit tests
cargo test --package storystream-feed-parser

# Run with output
cargo test --package storystream-feed-parser -- --nocapture

# Integration tests only
cargo test --package storystream-feed-parser --test parser_tests
```

**Test Coverage:**
- ✅ RSS 2.0 parsing (minimal, complex, with enclosures)
- ✅ Atom 1.0 parsing (minimal, complex, with links)
- ✅ Date parsing (RFC 2822 and RFC 3339)
- ✅ Audio enclosure detection
- ✅ Feed sorting
- ✅ Error handling (invalid XML, missing fields)
- ✅ Special characters and HTML entities
- ✅ Multiple audio formats (MP3, OGG, M4A)

## Performance

- **Parsing Speed**: ~1ms for typical podcast feed (20 items)
- **Memory Usage**: ~50KB per 100-item feed
- **Dependencies**: Minimal - only 4 direct dependencies

## Integration with StoryStream

The feed parser integrates with other StoryStream components:

```rust
use storystream_feed_parser::FeedParser;
use storystream_network::Downloader;
use storystream_library::LibraryManager;

// 1. Parse feed
let feed = FeedParser::parse(&rss_content)?;

// 2. Download audio files
let downloader = Downloader::new();
for item in feed.audio_items() {
    if let Some(url) = item.audio_url() {
        let path = downloader.download(url).await?;
        
        // 3. Import to library
        library.import_file(&path).await?;
    }
}
```

## Common Use Cases

### Podcast Manager

```rust
struct PodcastManager {
    feeds: Vec<String>,
}

impl PodcastManager {
    async fn check_for_new_episodes(&self) -> Result<Vec<FeedItem>> {
        let mut new_items = Vec::new();
        
        for feed_url in &self.feeds {
            let content = fetch(feed_url).await?;
            let feed = FeedParser::parse(&content)?;
            
            for item in feed.audio_items() {
                if is_new(item) {
                    new_items.push(item.clone());
                }
            }
        }
        
        Ok(new_items)
    }
}
```

### Audiobook Discovery

```rust
// Search LibriVox for specific author
async fn find_audiobooks(author: &str) -> Result<Vec<FeedItem>> {
    let search_url = format!(
        "https://librivox.org/search?q={}&format=rss",
        author
    );
    
    let content = fetch(&search_url).await?;
    let feed = FeedParser::parse(&content)?;
    
    Ok(feed.audio_items().into_iter().cloned().collect())
}
```

## Limitations

- **iTunes Extensions**: Not currently parsed (planned for v2)
- **Media RSS**: Not supported
- **Namespaces**: Only default RSS/Atom namespaces
- **Binary Data**: No inline base64 audio support

## Future Enhancements

- [ ] iTunes podcast extensions (itunes:duration, etc.)
- [ ] Media RSS namespace support
- [ ] Async parsing for large feeds
- [ ] Feed validation and sanitization
- [ ] Custom namespace handling

## Dependencies

```toml
[dependencies]
thiserror = "2.0.17"       # Error handling
serde = "1.0"              # Serialization
quick-xml = "0.36"         # Fast XML parsing
chrono = "0.4"             # Date/time handling
```

## License

Same as StoryStream project (AGPL-3.0-or-later OR Commercial)

## See Also

- [StoryStream Main Documentation](../../README.md)
- [Content Sources Module](../content-sources/README.md)
- [Network Module](../network/README.md)