// FILE: crates/media-formats/examples/analyze_audio.rs

use std::env;
use std::path::PathBuf;
use std::process;

use storystream_media_formats::{AudioAnalyzer, FormatError};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <audio_file>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} audiobook.flac", args[0]);
        process::exit(1);
    }

    let path = PathBuf::from(&args[1]);

    println!("🎵 StoryStream Audio Analyzer");
    println!("═══════════════════════════════\n");
    println!("📂 Analyzing: {}\n", path.display());

    let analyzer = match AudioAnalyzer::new() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("❌ Failed to create analyzer: {}", e);
            process::exit(1);
        }
    };

    match analyzer.analyze(&path) {
        Ok(props) => {
            println!("Audio File Properties");
            println!("=====================");
            println!("Format: {}", props.format);
            println!("Codec: {} ({})", props.codec.name, props.codec.codec_type);
            println!();

            println!("{}", props.quality.report());
            println!();

            // Display codec details
            println!("Codec Details");
            println!("=============");
            println!("Name: {}", props.codec.name);
            println!("Type: {}", props.codec.codec_type);
            println!("Lossless: {}", if props.codec.is_lossless { "Yes" } else { "No" });

            // Bitrate is on AudioProperties, not CodecInfo
            if let Some(bitrate) = props.bitrate {
                println!("Bitrate: {} kbps", bitrate / 1000);
            }
            println!();

            let caps = props.format.capabilities();
            println!("Format Capabilities");
            println!("==================");
            println!("Metadata Support: {}", if caps.metadata { "Yes" } else { "No" });
            println!("Cover Art: {}", if caps.cover_art { "Yes" } else { "No" });
            println!("Chapters: {}", if caps.chapters { "Yes" } else { "No" });
            println!("Streaming: {}", if caps.streaming { "Yes" } else { "No" });
            println!("Seekable: {}", if caps.seekable { "Yes" } else { "No" });
            println!();

            if props.quality.score() >= 90 {
                println!("✨ Audiophile Quality Detected!");
            } else if props.quality.score() >= 75 {
                println!("✓ High Quality Audio");
            } else {
                println!("ℹ Standard Quality Audio");
            }
        }
        Err(e) => {
            eprintln!("❌ Analysis failed: {}", e);

            match e {
                FormatError::FileNotFound { path } => {
                    eprintln!("\nFile not found: {}", path.display());
                    eprintln!("Please check that the file exists and is accessible.");
                }
                FormatError::UnsupportedFormat(format) => {
                    eprintln!("\nUnsupported format: {}", format);
                    eprintln!("StoryStream supports: MP3, M4B, M4A, FLAC, Opus, WAV, AIFF, OGG");
                }
                FormatError::UnsupportedFormatWithPath { format, path } => {
                    eprintln!("\nUnsupported format: {} in file {}", format, path.display());
                    eprintln!("StoryStream supports: MP3, M4B, M4A, FLAC, Opus, WAV, AIFF, OGG");
                }
                FormatError::CorruptedFile { path, reason } => {
                    eprintln!("\nCorrupted file: {}", path.display());
                    eprintln!("Reason: {}", reason);
                }
                FormatError::ReadError { path, reason } => {
                    eprintln!("\nFailed to read: {}", path.display());
                    eprintln!("Reason: {}", reason);
                }
                _ => {
                    eprintln!("\nDetailed error: {:?}", e);
                }
            }

            process::exit(1);
        }
    }
}