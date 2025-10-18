// crates/feed-parser/examples/parse_feed.rs
//! Example of parsing RSS and Atom feeds

use storystream_feed_parser::FeedParser;

fn main() {
    println!("=== StoryStream Feed Parser Demo ===\n");

    // Example 1: Parse RSS feed
    println!("Example 1: RSS 2.0 Feed");
    println!("{}", "=".repeat(60));
    parse_rss_example();

    println!("\n");

    // Example 2: Parse Atom feed
    println!("Example 2: Atom Feed");
    println!("{}", "=".repeat(60));
    parse_atom_example();

    println!("\n");

    // Example 3: Filter and sort
    println!("Example 3: Filtering & Sorting");
    println!("{}", "=".repeat(60));
    filter_sort_example();
}

fn parse_rss_example() {
    let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Classic Audiobooks</title>
    <description>Public domain audiobooks read by volunteers</description>
    <link>https://example.com/audiobooks</link>
    <language>en</language>

    <item>
      <title>Pride and Prejudice - Chapter 1</title>
      <description>By Jane Austen. Read by volunteer narrator.</description>
      <link>https://example.com/pride-ch1</link>
      <guid>pride-ch1</guid>
      <pubDate>Mon, 01 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="https://example.com/audio/pride-ch1.mp3"
                 type="audio/mpeg"
                 length="15000000"/>
      <author>Jane Austen</author>
    </item>

    <item>
      <title>Pride and Prejudice - Chapter 2</title>
      <description>Continuing the classic novel.</description>
      <pubDate>Tue, 02 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="https://example.com/audio/pride-ch2.mp3"
                 type="audio/mpeg"
                 length="14500000"/>
    </item>

    <item>
      <title>Moby Dick - Chapter 1: Loomings</title>
      <description>By Herman Melville. Call me Ishmael...</description>
      <pubDate>Wed, 03 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="https://example.com/audio/moby-ch1.mp3"
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
            println!("{}", "-".repeat(60));

            for (i, item) in feed.items.iter().enumerate() {
                println!("\n{}. {}", i + 1, item.title);

                if let Some(desc) = &item.description {
                    let short_desc = if desc.len() > 50 {
                        format!("{}...", &desc[..50])
                    } else {
                        desc.clone()
                    };
                    println!("   Description: {}", short_desc);
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
                        println!("   Size: {:.1} MB", size as f64 / 1_000_000.0);
                    }
                }
            }
        }
        Err(e) => eprintln!("Error parsing RSS: {}", e),
    }
}

fn parse_atom_example() {
    let atom = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Science Fiction Audiobooks</title>
  <subtitle>Classic sci-fi stories</subtitle>
  <link href="https://example.com/scifi"/>

  <entry>
    <title>The Time Machine - Part 1</title>
    <link href="https://example.com/timemachine1"/>
    <id>timemachine-1</id>
    <published>2024-01-10T12:00:00Z</published>
    <summary>H.G. Wells' masterpiece about time travel.</summary>
  </entry>

  <entry>
    <title>The Time Machine - Part 2</title>
    <link href="https://example.com/timemachine2"/>
    <id>timemachine-2</id>
    <published>2024-01-11T12:00:00Z</published>
    <summary>Continuing the journey through time.</summary>
  </entry>

  <entry>
    <title>War of the Worlds - Chapter 1</title>
    <link href="https://example.com/wotw1"/>
    <id>wotw-1</id>
    <published>2024-01-12T12:00:00Z</published>
    <summary>The coming of the Martians.</summary>
  </entry>
</feed>"#;

    match FeedParser::parse(atom) {
        Ok(feed) => {
            println!("Feed: {}", feed.title);
            println!("Type: Atom");

            if let Some(desc) = &feed.description {
                println!("Description: {}", desc);
            }

            println!("\nEntries: {}", feed.item_count());
            println!("{}", "-".repeat(60));

            for (i, item) in feed.items.iter().enumerate() {
                println!("\n{}. {}", i + 1, item.title);

                if let Some(url) = &item.url {
                    println!("   Link: {}", url);
                }

                if let Some(summary) = &item.description {
                    println!("   Summary: {}", summary);
                }

                if let Some(published) = &item.published {
                    println!("   Published: {}", published.format("%B %d, %Y"));
                }
            }
        }
        Err(e) => eprintln!("Error parsing Atom: {}", e),
    }
}

fn filter_sort_example() {
    let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Mixed Content Feed</title>

    <item>
      <title>Audio Episode 1</title>
      <pubDate>Mon, 01 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="https://example.com/audio1.mp3" type="audio/mpeg"/>
    </item>

    <item>
      <title>Text Article</title>
      <pubDate>Tue, 02 Jan 2024 12:00:00 GMT</pubDate>
    </item>

    <item>
      <title>Audio Episode 2</title>
      <pubDate>Wed, 03 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="https://example.com/audio2.mp3" type="audio/mpeg"/>
    </item>

    <item>
      <title>Video Content</title>
      <pubDate>Thu, 04 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="https://example.com/video.mp4" type="video/mp4"/>
    </item>

    <item>
      <title>Audio Episode 3</title>
      <pubDate>Fri, 05 Jan 2024 12:00:00 GMT</pubDate>
      <enclosure url="https://example.com/audio3.mp3" type="audio/mpeg"/>
    </item>
  </channel>
</rss>"#;

    match FeedParser::parse(rss) {
        Ok(mut feed) => {
            println!("Original feed: {} items", feed.item_count());

            // Filter to audio only
            let audio_items = feed.audio_items();
            println!("Audio items: {}", audio_items.len());

            for item in audio_items {
                println!("  - {}", item.title);
                if let Some(url) = item.audio_url() {
                    println!("    URL: {}", url);
                }
            }

            println!("\nSorting by date (newest first)...");
            feed.sort_by_date();

            println!("Sorted items:");
            for (i, item) in feed.items.iter().enumerate() {
                print!("  {}. {}", i + 1, item.title);
                if let Some(date) = &item.published {
                    print!(" ({})", date.format("%b %d"));
                }
                println!();
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}