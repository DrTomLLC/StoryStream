# 🎧 StoryStream - Professional Audiobook Player

A modern, feature-rich audiobook player written in Rust with support for multiple platforms, cross-device synchronization, and a beautiful terminal interface.

[![Tests](https://img.shields.io/badge/tests-750%2B%20passing-brightgreen)](.)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org)

## ✨ Features

### 🎵 **Complete Audio Engine**
- Multiple format support (MP3, M4B, FLAC, OGG, WAV, AAC, OPUS)
- Variable playback speed (0.5x - 3.0x) with pitch correction
- Chapter navigation and management
- Bookmark creation and organization
- Equalizer with presets (Flat, Bass Boost, Voice Boost)
- Sleep timer with fade-out
- Resume from last position

### 📚 **Library Management**
- Automatic library scanning
- Metadata extraction from audio files
- Full-text search across titles, authors, and narrators
- Smart playlists based on criteria
- Favorite books tracking
- Progress tracking and statistics
- Cover art support

### 🔄 **Cross-Device Sync**
- Synchronize playback positions across devices
- Bookmark syncing
- Library metadata sync
- Intelligent conflict resolution
- Offline support with sync queuing

### 🌐 **Content Discovery**
- Download from LibriVox (public domain audiobooks)
- Internet Archive integration
- RSS/Atom podcast feed support
- Local file import

### 💾 **Data Management**
- SQLite database for persistence
- Automatic backups
- Configuration management with validation
- Migration system for upgrades

### 🎨 **User Interfaces**
- **TUI**: Beautiful terminal interface
- **CLI**: Command-line interface for automation
- **API**: Library for custom integrations

### 🛡️ **Enterprise-Grade Quality**
- 750+ comprehensive tests
- Zero `unwrap()` calls - explicit error handling
- Graceful degradation
- Retry logic with exponential backoff
- Circuit breaker pattern for resilience
- Full documentation

## 📦 Installation

### Prerequisites

- Rust 1.75 or later
- SQLite 3.35 or later

### Build from Source
```bash
# Clone the repository
git clone https://github.com/yourusername/storystream.git
cd storystream

# Build release version
cargo build --release

# Install
cargo install --path crates/cli
```

### Quick Start
```bash
# Scan your audiobook library
storystream scan ~/Audiobooks

# Play an audiobook
storystream play "Moby Dick"

# Launch the TUI
cargo run --example tui_demo
```

## 🚀 Quick Start Guide

### 1. Set up your library
```bash
# Add library paths
storystream config set library.paths ~/Audiobooks /media/audiobooks

# Scan for audiobooks
storystream scan

# View your library
storystream list
```

### 2. Play an audiobook
```bash
# Play by title
storystream play "Pride and Prejudice"

# Resume last played
storystream resume

# Control playback
storystream pause
storystream play
storystream seek +30s
storystream speed 1.5x
```

### 3. Use bookmarks
```bash
# Add a bookmark
storystream bookmark "Important quote"

# List bookmarks
storystream bookmarks list

# Jump to a bookmark
storystream bookmarks goto "Important quote"
```

### 4. Launch the TUI
```bash
# Start the terminal UI
cargo run --example tui_demo

# Navigate with:
# - Tab: Switch views
# - Arrow keys: Navigate
# - Space: Play/Pause
# - h: Help
```

## 📖 Documentation

### User Guides
- [TUI User Guide](crates/tui/README.md) - Terminal interface guide
- [CLI Reference](docs/CLI.md) - Command-line usage
- [Configuration Guide](docs/CONFIGURATION.md) - Settings and customization

### Developer Documentation
- [Architecture Overview](docs/ARCHITECTURE.md) - System design
- [API Documentation](docs/API.md) - Library usage
- [Contributing Guide](CONTRIBUTING.md) - Development guidelines

### Module Documentation
- [Media Engine](crates/media-engine/README.md) - Audio playback
- [Sync Engine](crates/sync-engine/README.md) - Cross-device sync
- [Database](crates/database/README.md) - Data persistence
- [Network](crates/network/README.md) - HTTP client

## 🎯 Usage Examples

### Command Line
```bash
# Basic playback
storystream play "1984.m4b"

# With options
storystream play "1984.m4b" --speed 1.5 --volume 80

# Chapter navigation
storystream chapter next
storystream chapter goto 5

# Library management
storystream import ~/Downloads/audiobook.m4b
storystream search "Orwell"
storystream stats

# Download from LibriVox
storystream download --source librivox "Pride and Prejudice"

# Import podcast feed
storystream import-feed https://example.com/audiobooks.rss
```

### Rust API
```rust
use media_engine::MediaEngine;
use storystream_config::ConfigManager;

// Initialize
let config = ConfigManager::load_or_default()?;
let mut engine = MediaEngine::new(config.player)?;

// Load and play
engine.load("audiobook.m4b")?;
engine.play()?;

// Control playback
engine.set_speed(1.5)?;
engine.seek(std::time::Duration::from_secs(30))?;

// Add bookmark
engine.bookmarks().add_auto()?;
```

### Sync Integration
```rust
use storystream_sync_engine::{SyncEngine, SyncConfig};

// Set up sync
let config = SyncConfig::default();
let engine = SyncEngine::new(config);

// Record a change
engine.record_change(
    ChangeType::Update,
    EntityType::Position,
    "book-123".to_string(),
    serde_json::json!({"position": 1000}),
)?;

// Sync with server
let request = engine.create_sync_request()?;
// Send request to server...
let response = server.sync(request)?;
engine.process_sync_response(response)?;
```

## 🏗️ Architecture

StoryStream is built with a modular, crate-based architecture:
```
StoryStream/
├── crates/
│   ├── cli/              # Command-line interface
│   ├── tui/              # Terminal UI
│   ├── core/             # Core types and errors
│   ├── media-engine/     # Audio playback engine
│   ├── config/           # Configuration management
│   ├── database/         # SQLite persistence
│   ├── library/          # Library management
│   ├── sync-engine/      # Cross-device sync
│   ├── network/          # HTTP client
│   ├── resilience/       # Retry and circuit breaker
│   ├── media-formats/    # Format detection
│   ├── content-sources/  # LibriVox, Archive.org
│   ├── feed-parser/      # RSS/Atom parsing
│   ├── android-bridge/   # Android FFI (stub)
│   └── wear-bridge/      # Wear OS (stub)
└── docs/                 # Documentation
```

### Module Overview

| Module | Purpose | Tests | Status |
|--------|---------|-------|--------|
| **media-engine** | Audio playback, chapters, bookmarks | 150 | ✅ Complete |
| **config** | Configuration management | 122 | ✅ Complete |
| **core** | Core types, errors, domain models | 153 | ✅ Complete |
| **database** | SQLite persistence, migrations | 35 | ✅ Complete |
| **library** | Library scanning, metadata | 28 | ✅ Complete |
| **sync-engine** | Cross-device synchronization | 45 | ✅ Complete |
| **network** | HTTP client, downloads | 33 | ✅ Complete |
| **resilience** | Retry, circuit breaker | 32 | ✅ Complete |
| **tui** | Terminal user interface | 41 | ✅ Complete |
| **cli** | Command-line interface | 9 | ✅ Complete |
| **Total** | | **750+** | **🎉** |

## 🧪 Testing
```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --package media-engine
cargo test --package storystream-tui

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## 🎨 Configuration

Configuration file location:
- **Linux/Mac**: `~/.config/storystream/config.toml`
- **Windows**: `%APPDATA%\storystream\config.toml`

Example configuration:
```toml
[app]
version = "1.0.0"
log_level = "Info"
color_scheme = "Dark"

[library]
paths = ["~/Audiobooks", "/media/audiobooks"]
auto_scan = true
scan_interval = 3600

[player]
default_volume = 1.0
default_speed = 1.0
auto_save_interval = 30
resume_on_start = true

[sync]
enabled = false
auto_sync = false
conflict_resolution = "UseNewest"
```

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup
```bash
# Clone and build
git clone https://github.com/yourusername/storystream.git
cd storystream
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Areas for Contribution

- [ ] Mobile apps (Android, iOS)
- [ ] Web interface
- [ ] Additional content sources
- [ ] More TUI themes
- [ ] Plugin system
- [ ] Cloud backup integration

## 📊 Project Status

| Feature | Status |
|---------|--------|
| Core Audio Engine | ✅ Complete |
| Library Management | ✅ Complete |
| Cross-Device Sync | ✅ Complete |
| Terminal UI | ✅ Complete |
| CLI Interface | ✅ Complete |
| Configuration | ✅ Complete |
| Database | ✅ Complete |
| Network Layer | ✅ Complete |
| Content Discovery | ✅ Complete |
| Android App | ⏳ Planned |
| iOS App | ⏳ Planned |
| Web Interface | ⏳ Planned |

**Overall: 85% Complete**

## 🎯 Roadmap

### v1.0 (Current)
- [x] Core audio engine
- [x] Library management
- [x] Terminal UI
- [x] Cross-device sync
- [x] LibriVox integration

### v1.1 (Planned)
- [ ] Web interface
- [ ] Cloud backup
- [ ] Advanced search
- [ ] Statistics dashboard

### v2.0 (Future)
- [ ] Mobile apps
- [ ] Social features
- [ ] Podcast management
- [ ] Plugin system

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Built with these excellent libraries:

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [rodio](https://github.com/RustAudio/rodio) - Audio playback
- [cpal](https://github.com/RustAudio/cpal) - Cross-platform audio
- [sqlx](https://github.com/launchbadge/sqlx) - SQL toolkit
- [tokio](https://tokio.rs/) - Async runtime

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/storystream/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/storystream/discussions)
- **Email**: support@storystream.example.com

## ⭐ Star History

If you find StoryStream useful, please consider starring the repository!

---

**Made with ❤️ and 🦀 Rust**