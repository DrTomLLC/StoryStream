// crates/feed-parser/examples/parse_feed.rs
//! Example of parsing an RSS feed

use storystream_feed_parser::FeedParser;

fn main() {
    let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Classic Audiobooks</title>
    <description>Public domain audiobooks read by volunteers</description>
    <link>http://example.com/audiobooks</link>
    <language>en</language>

    <item>
      <title>Pride and Prejudice - Chapter 1</title>
      <description>By Jane Austen. Read by volunteer narrator.</description>
      <link>http://example.com/pride-ch1</link>
      <guid>pride-ch1</guid>
      <pubDate>Mon, 01 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="http://example.com/audio/pride-ch1.mp3"
                 type="audio/mpeg"
                 length="15000000"/>
      <author>Jane Austen</author>
    </item>

    <item>
      <title>Pride and Prejudice - Chapter 2</title>
      <description>Continuing the classic novel.</description>
      <pubDate>Tue, 02 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="http://example.com/audio/pride-ch2.mp3"
                 type="audio/mpeg"
                 length="14500000"/>
    </item>

    <item>
      <title>Moby Dick - Chapter 1: Loomings</title>
      <description>By Herman Melville. Call me Ishmael...</description>
      <pubDate>Wed, 03 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="http://example.com/audio/moby-ch1.mp3"
                 type="audio/mpeg"
                 length="18000000"/>
      <author>Herman Melville</author>
    </item>
  </channel>
</rss>"#;

    match FeedParser::parse(rss) {
        Ok(feed) => {
            println!("Feed: {}", feed.title);

            if let Some(desc) = &feed.description {
                println!("Description: {}", desc);
            }

            if let Some(url) = &feed.url {
                println!("URL: {}", url);
            }

            if let Some(lang) = &feed.language {
                println!("Language: {}", lang);
            }

            println!("\nItems: {}", feed.item_count());
            println!("{}", "=".repeat(60));

            for (i, item) in feed.items.iter().enumerate() {
                println!("\n{}. {}", i + 1, item.title);

                if let Some(desc) = &item.description {
                    println!("   Description: {}", desc);
                }

                if let Some(author) = &item.author {
                    println!("   Author: {}", author);
                }

                if let Some(published) = &item.published {
                    println!("   Published: {}", published.format("%Y-%m-%d"));
                }

                if let Some(enclosure) = &item.enclosure {
                    println!("   Audio: {}", enclosure.url);
                    if let Some(size) = enclosure.length {
                        println!("   Size: {} MB", size / 1_000_000);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing feed: {}", e);
            std::process::exit(1);
        }
    }
}