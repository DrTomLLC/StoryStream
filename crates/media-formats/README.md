# StoryStream Media Formats

**High-quality audio format detection and analysis for StoryStream audiobook player.**

## Features

### 🎵 Comprehensive Format Support
- **Lossless**: FLAC, ALAC, APE, WavPack, TTA
- **Uncompressed**: WAV, AIFF
- **High-Quality Lossy**: Opus, Vorbis, AAC (M4A/M4B)
- **Legacy Lossy**: MP3, WMA
- **Containers**: Matroska Audio, WebM

### 🔍 Detailed Analysis
- Sample rate and bit depth detection
- Duration calculation
- Bitrate analysis (CBR/VBR detection)
- Channel configuration
- Codec information extraction

### 📊 Quality Classification
- Automatic quality tier detection (Low → Studio)
- Audiophile quality identification (Hi-Res 96kHz, 192kHz+)
- Compression ratio calculation
- Quality scoring (0-100)

### ⚠️ Zero Panics Guarantee
- All operations return `Result` types
- Comprehensive error handling
- No `unwrap()`, `expect()`, or `panic!()` in production code

## Usage

### Basic Format Detection

```rust
use storystream_media_formats::AudioFormat;
use std::path::Path;

let path = Path::new("audiobook.flac");
let format = AudioFormat::from_path(path);

if let Some(format) = format {
    println!("Format: {}", format);
    println!("Lossless: {}", format.is_lossless());
    println!("Supports chapters: {}", format.supports_chapters());
}
```

### Complete File Analysis

```rust
use storystream_media_formats::AudioAnalyzer;
use std::path::Path;

let analyzer = AudioAnalyzer::new()?;
let properties = analyzer.analyze(Path::new("audiobook.flac"))?;

println!("Format: {}", properties.format);
println!("Quality: {}", properties.quality_tier);
println!("Sample Rate: {} Hz", properties.sample_rate);
println!("Bit Depth: {} bits", properties.bits_per_sample);
println!("Duration: {:?}", properties.duration);
println!("Audiophile: {}", properties.is_audiophile());
```

### Quality Assessment

```rust
use storystream_media_formats::AudioQuality;

let quality = AudioQuality::new(
    96_000,  // 96kHz sample rate
    24,      // 24-bit depth
    true,    // Lossless
    false,   // Not uncompressed
    None,    // No bitrate (lossless)
);

println!("{}", quality.report());
println!("Score: {}/100", quality.score());
println!("Is Hi-Res: {}", quality.tier.is_audiophile());
```

## Quality Tiers

| Tier | Description | Typical Use |
|------|-------------|-------------|
| **Low** | < 128 kbps | Voice, podcasts |
| **Standard** | 128-192 kbps | General listening |
| **High** | 192-320 kbps | High-quality lossy |
| **CD** | 16-bit/44.1kHz | CD-quality lossless |
| **DVD** | 16-bit/48kHz | DVD-quality |
| **Hi-Res 96** | 24-bit/96kHz | Audiophile quality |
| **Hi-Res 192** | 24-bit/192kHz+ | Ultra hi-res |
| **Studio** | 32-bit float | Studio masters |

## Format Capabilities

Each format has different capabilities:

```rust
let caps = AudioFormat::Flac.capabilities();

assert!(caps.metadata);       // ID3/Vorbis comments
assert!(caps.cover_art);      // Embedded artwork
assert!(caps.chapters);       // Chapter markers
assert!(caps.lossless);       // Lossless compression
assert!(caps.seekable);       // Random access
```

## Error Handling

All operations return `Result<T, FormatError>`:

```rust
use storystream_media_formats::{AudioAnalyzer, FormatError};

match analyzer.analyze(path) {
    Ok(props) => {
        // Success
    }
    Err(FormatError::FileNotFound { .. }) => {
        // Handle missing file
    }
    Err(FormatError::UnsupportedFormat { format, .. }) => {
        // Handle unsupported format
    }
    Err(FormatError::CorruptedFile { path, reason }) => {
        // Handle corrupted file
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## Examples

Run the audio analyzer example:

```bash
cargo run --example analyze_audio -- /path/to/audiobook.flac
```

Output:
```
🎵 StoryStream Audio Analyzer
═══════════════════════════════

📂 Analyzing: audiobook.flac

Audio File Properties
=====================
Format: FLAC
Codec: FLAC

Quality: Hi-Res 96kHz (Hi-Res audio (24-bit/96kHz))
Sample Rate: 96000 Hz
Bit Depth: 24 bits
Compression: Lossless
Duration: 5:32:15
Quality Score: 97/100

Channels: 2
File Size: 1.23 GB
Compression Ratio: 2.1:1

✨ Audiophile Quality Detected!
```

## Architecture

### Zero-Panic Design

- **No unwrap()**: All fallible operations return `Result`
- **No expect()**: Errors are properly handled
- **No panic!()**: Even invalid input produces error results
- **Lock-free**: No mutex poisoning issues

### Performance

- **Lazy Analysis**: Only analyzes when requested
- **Efficient Probing**: Uses Symphonia's optimized format detection
- **Memory Efficient**: Streams file data, doesn't load entire files

### Extensibility

Add new formats by:
1. Adding variant to `AudioFormat` enum
2. Adding extension mapping
3. Implementing format capabilities
4. Adding tests

## Testing

Run all tests:
```bash
cargo test --package storystream-media-formats
```

Run with coverage:
```bash
cargo tarpaulin --package storystream-media-formats
```

## Dependencies

- **symphonia**: High-quality audio decoding
- **thiserror**: Error handling
- **serde**: Serialization support

## License

AGPL-3.0-or-later OR Commercial

## Integration with StoryStream

This crate is used by:
- `media-engine`: Audio playback engine
- `library`: Book importing and metadata extraction
- `cli`: Format detection and validation

## Future Enhancements

- [ ] Magic byte detection (beyond extension)
- [ ] DRM detection
- [ ] More detailed codec analysis
- [ ] Audio fingerprinting
- [ ] Embedded cue sheet support
- [ ] Multi-file audiobook detection