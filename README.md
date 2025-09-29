# StoryStream 🎧📚

**Open Source Audiobook & Podcast Player for Android and Wear OS**

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](LICENSE-AGPL-3.0.txt)
[![Commercial License Available](https://img.shields.io/badge/License-Commercial-green.svg)](LICENSE-COMMERCIAL.txt)
[![Rust 1.90+](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org)
[![Android 11+](https://img.shields.io/badge/Android-11%2B-green.svg)](https://developer.android.com)
[![Build Status](https://github.com/DrTomLLC/StoryStream/workflows/CI/badge.svg)](https://github.com/DrTomLLC/StoryStream/actions)

---

## 🌟 What is StoryStream?

StoryStream is a **privacy-respecting, ad-free** audiobook and podcast player built from the ground up with modern technology. Unlike commercial apps filled with ads and tracking, StoryStream puts you in control of your audio library.

### Why StoryStream?

- **🚫 No Ads, Ever** - Built to respect your time and attention
- **🔒 Privacy First** - Your data stays on your device
- **🎵 Universal Format Support** - Plays everything (MP3, M4B, M4A, FLAC, OGG, Opus, AAC, WAV)
- **📡 Streaming & Local** - Download or stream from multiple sources
- **🌐 Open Source** - Community-driven, transparent development
- **⚡ Future-Proof** - Rust core enables iOS and desktop ports
- **📖 Rich Content** - Integrated LibriVox and Internet Archive access
- **🎚️ Advanced Features** - Variable speed, skip silence, EQ, sleep timer

---

## ✨ Features

### Current (Phase 1 - MVP)

#### **Playback Engine**
- ✅ All major audio formats (MP3, M4B, M4A, FLAC, OGG, Opus, AAC, WAV, AIFF)
- ✅ Streaming with progressive download and intelligent caching
- ✅ Variable playback speed (0.5x - 3.0x) with pitch correction
- ✅ Skip silence detection
- ✅ 10-band equalizer with presets
- ✅ Volume boost for quiet recordings
- ✅ Sleep timer with smooth fade-out
- ✅ Unlimited bookmarks per audiobook
- ✅ Automatic position saving (every 5 seconds)

#### **Library Management**
- ✅ Auto-scan device storage (recursive, background)
- ✅ Manual file/folder import
- ✅ Comprehensive metadata extraction (all formats)
- ✅ Embedded cover art display
- ✅ Manual metadata editing
- ✅ Smart organization by series, author, narrator
- ✅ Full-text search across titles, authors, descriptions
- ✅ Playlists and smart playlists
- ✅ Recently played and favorites
- ✅ Support for 50,000-100,000+ audiobook libraries

#### **Content Sources**
- ✅ Local files (your audiobooks)
- ✅ **LibriVox** - 13,000+ free public domain audiobooks
- ✅ **Internet Archive** - Massive audiobook collection
- ✅ Manual URL imports

#### **User Experience**
- ✅ Material 3 design with dynamic theming
- ✅ Dark and light modes
- ✅ Grid and list library views
- ✅ Lock screen controls
- ✅ Android notification controls
- ✅ Background playback
- ✅ Bluetooth controls (car stereo, headphones)

#### **Data Management**
- ✅ Hybrid storage (app-specific + shared storage)
- ✅ Export library (full backup to JSON)
- ✅ Import library (restore from backup)
- ✅ Automatic database backups (daily)
- ✅ Triple redundancy for critical data
- ✅ Automatic crash recovery

### Coming Soon (Phase 2) - Target: +2 months

- 🔜 **Podcast Support**
  - RSS feed subscriptions
  - Automatic episode downloads
  - Background feed updates
  - OPML import/export
  - Podcast discovery/search

- 🔜 **Enhanced Media Features**
  - Chapter images synchronized with playback
  - Enhanced audiobook cover art management
  - Playlist enhancements
  - Advanced search filters

### Future (Phase 3) - Target: +4 months

- 📅 **Wear OS App**
  - Standalone watch app
  - Phone ↔ Watch sync
  - Offline playback on watch
  - Watch controls and complications

- 📅 **Cross-Device Sync**
  - Real-time playback position sync
  - Library sync across devices
  - Conflict resolution
  - User-provided cloud storage support (WebDAV, S3, Nextcloud, etc.)

### Vision (Phase 4) - Target: +7 months

- 🔮 **Rich Media Stories**
  - Video chapter playback
  - Synchronized graphics/text overlays
  - Custom story format support
  - Content creation tools (separate module)

- 🔮 **Cloud Sync Infrastructure**
  - Multiple backend support (WebDAV, S3, SFTP, custom)
  - End-to-end encryption
  - Bandwidth-aware syncing

- 🔮 **Android Auto**
  - Full Android Auto integration
  - Voice commands
  - Car-optimized UI

---

## 🚀 Quick Start

### For Users

**Requirements:**
- Android 11 (API 30) or newer
- Galaxy S21 or newer recommended
- 500 MB free storage minimum

**Installation:**

1. **Download Latest Release**
   - Visit [Releases](https://github.com/DrTomLLC/StoryStream/releases)
   - Download `StoryStream-v0.1.0.apk`

2. **Install APK**
   - Enable "Install from Unknown Sources" in Android settings
   - Open the downloaded APK
   - Follow installation prompts

3. **Grant Permissions**
   - Storage access (for your audiobook library)
   - Notifications (for playback controls)

4. **Start Listening**
   - Add audiobooks from your device
   - Browse LibriVox catalog
   - Search Internet Archive

### For Developers

**Requirements:**
- Windows 11 (primary development environment)
- Rust 1.90.0+ stable
- Android Studio (latest)
- Android SDK API 30+ with NDK
- JetBrains RustRover (recommended)
- Git

**Setup:**

```batch
# 1. Clone repository
git clone https://github.com/DrTomLLC/StoryStream.git
cd StoryStream

# 2. Run Windows setup script
scripts\setup-windows.bat

# 3. Install Rust Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android

# 4. Install cargo tools
cargo install cargo-ndk
cargo install uniffi-bindgen
cargo install sqlx-cli --no-default-features --features sqlite

# 5. Build debug version
scripts\build-debug.bat

# 6. Run tests
scripts\run-tests.bat
```

**Detailed Setup Guide:** See [docs/WINDOWS_SETUP.md](docs/WINDOWS_SETUP.md)

---

## 🏗️ Architecture

### Technology Stack

**Core (95% - Rust)**
- Business logic, media engine, sync, library management
- **Database:** SQLx with SQLite (async, type-safe)
- **Audio:** Symphonia (multi-format decoder)
- **Networking:** reqwest + tokio (async HTTP)
- **Metadata:** id3, mp4ameta, metaflac (comprehensive parsing)

**UI (5% - Kotlin)**
- Jetpack Compose with Material 3
- Static templates driven by Rust events
- Android MediaSession integration

**Bridge**
- uniffi (Rust ↔ Kotlin FFI)
- Event-driven architecture
- Type-safe, auto-generated bindings

### Design Principles

1. **Rust Core, Kotlin Shell**
   - 95% of logic in Rust (portable, testable)
   - Kotlin handles only Android-specific UI and system APIs
   - Kotlin templates rarely change after Phase 1

2. **Future-Proof**
   - Business logic portable to iOS, desktop, web
   - Database migrations handle schema evolution
   - Feature flags for gradual rollouts

3. **Maximum Stability**
   - Triple data redundancy (DB + JSON + cloud)
   - Automatic crash recovery
   - Graceful degradation for failures
   - Comprehensive error handling

4. **Modular & Extensible**
   - Trait-based content sources (drop-in new sources)
   - Dynamic configuration (settings in DB)
   - Plugin architecture for features
   - Per-crate tests and fixtures

5. **Privacy & Performance**
   - Zero telemetry by default
   - Local-first architecture
   - Optimized for 50k-100k libraries
   - <150MB memory usage
   - <1.5s cold start time

### Project Structure

```
StoryStream/
├── crates/           # Rust workspace (10+ crates)
│   ├── core/         # Error types, domain models
│   ├── database/     # SQLx + migrations
│   ├── media-*       # Audio processing
│   ├── library/      # Content management
│   ├── content-sources/  # LibriVox, Archive, etc.
│   ├── sync-engine/  # Cross-device sync
│   └── android-bridge/   # uniffi FFI
├── android/          # Kotlin UI (~400 lines)
├── docs/             # Documentation
├── scripts/          # Build scripts (Windows)
└── tests/            # Integration, stress, fuzz tests
```

**Detailed Architecture:** See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

---

## 🤝 Contributing

StoryStream is open source and welcomes contributions! However, all contributors must sign a Contributor License Agreement (CLA).

### Before Contributing

1. **Read the Contributing Guide:** [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)
2. **Review the Roadmap:** [docs/ROADMAP.md](docs/ROADMAP.md)
3. **Check Existing Issues:** [GitHub Issues](https://github.com/DrTomLLC/StoryStream/issues)
4. **Sign the CLA:** Required for all pull requests ([CLA.md](CLA.md))

### Contribution Process

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** changes using [Conventional Commits](https://www.conventionalcommits.org/)
   ```
   feat: Add LibriVox search filters
   fix: Resolve streaming cache corruption
   docs: Update build instructions
   test: Add stress tests for large libraries
   ```
4. **Test** thoroughly (run `scripts\run-tests.bat`)
5. **Sign CLA** (automatic check on first PR)
6. **Submit** pull request

### Good First Issues

Look for issues tagged [`good-first-issue`](https://github.com/DrTomLLC/StoryStream/labels/good-first-issue) - these are beginner-friendly and well-documented.

### Development Guidelines

- **Code Style:** Run `cargo fmt` before committing
- **Linting:** Ensure `cargo clippy` passes with no warnings
- **Tests:** All new features require tests (unit + integration)
- **Documentation:** Update docs for user-facing changes
- **Commits:** Use Conventional Commits format
- **PRs:** Keep focused (one feature/fix per PR)

---

## 📜 Licensing

StoryStream uses **dual licensing** to balance open source community and sustainable development:

### For Open Source Use: AGPL-3.0

**Free to use, modify, and distribute** under these conditions:
- ✅ Use for personal projects
- ✅ Use for open source projects
- ✅ Modify and distribute modifications
- ✅ Use in your organization
- ⚠️ **Must** keep source code open (AGPL-3.0)
- ⚠️ **Must** share modifications publicly
- ⚠️ Network use = must provide source code

See [LICENSE-AGPL-3.0.txt](LICENSE-AGPL-3.0.txt) for full terms.

### For Commercial/Proprietary Use: Commercial License

**Want to use StoryStream in closed-source products?**
- ✅ Keep modifications private
- ✅ No AGPL obligations
- ✅ Priority support
- ✅ Custom feature development

**Contact:** licensing@storystream.app (or open a discussion)

See [LICENSE-COMMERCIAL.txt](LICENSE-COMMERCIAL.txt) for details.

### Contributor License Agreement (CLA)

All contributors must sign the CLA ([CLA.md](CLA.md)), which:
- Grants StoryStream project rights to use your contributions
- Allows dual licensing of contributions
- Protects the project and contributors legally
- Standard practice (used by Linux, Google, Apache, etc.)

**Automated:** CLA Assistant will prompt you on your first PR.

---

## 🗺️ Roadmap

### Phase 1: MVP (Current) - Month 0-4
**Goal:** Solid audiobook player with streaming and content discovery

- [x] Core playback engine (all formats)
- [x] Library management (50k+ capacity)
- [x] LibriVox integration
- [x] Internet Archive integration
- [x] Local file support
- [x] Advanced playback features
- [x] Material 3 UI
- [x] Crash recovery
- [ ] Beta testing (Month 4)
- [ ] Initial release (Month 4)

### Phase 2: Podcasts & Enhancement - Month 4-6
**Goal:** Full podcast support and enhanced media features

- [ ] RSS feed subscriptions
- [ ] Podcast discovery
- [ ] Automatic downloads
- [ ] OPML import/export
- [ ] Chapter images
- [ ] Enhanced UI features
- [ ] Community feedback integration
- [ ] F-Droid release

### Phase 3: Wear OS & Sync - Month 6-8
**Goal:** Cross-device experience

- [ ] Wear OS standalone app
- [ ] Phone ↔ Watch communication
- [ ] Offline watch playback
- [ ] Playback position sync
- [ ] User-provided cloud backends
- [ ] Conflict resolution

### Phase 4: Rich Media & Cloud - Month 8-11
**Goal:** Advanced features and content creation

- [ ] Video chapter playback
- [ ] Rich story format
- [ ] Cloud sync infrastructure
- [ ] Content creation tools
- [ ] Android Auto support
- [ ] Chromecast support
- [ ] Google Play Store release

### Phase 5: Multi-Platform - Month 11+
**Goal:** Expand beyond Android

- [ ] iOS app (using same Rust core)
- [ ] Desktop app (Linux, Windows, macOS)
- [ ] Web interface
- [ ] Plugin system
- [ ] Multi-language support

**Detailed Roadmap:** [docs/ROADMAP.md](docs/ROADMAP.md)

---

## 🧪 Testing

StoryStream has comprehensive testing at multiple levels:

### Test Categories

**Unit Tests** (per-crate)
```batch
cargo test --workspace
```

**Integration Tests** (cross-crate workflows)
```batch
cargo test --test '*' 
```

**Stress Tests** (large libraries, long sessions)
```batch
cargo test --release --test stress_*
```

**Fuzz Tests** (random/corrupt inputs)
```batch
cargo +nightly fuzz run metadata_parser
```

### Test Coverage

Current coverage: **Target 80%+**

```batch
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

View coverage: `coverage/index.html`

### Continuous Integration

All PRs must pass:
- ✅ All unit tests
- ✅ All integration tests  
- ✅ Clippy with zero warnings
- ✅ Code formatting check
- ✅ Security audit
- ✅ Build for all Android architectures

**Testing Guide:** [docs/TESTING.md](docs/TESTING.md)

---

## 📊 Performance

### Benchmarks (Galaxy S22 Ultra)

| Metric | Target | Current |
|--------|--------|---------|
| Cold start | < 1.5s | ~1.2s ✅ |
| Memory usage | < 150MB | ~120MB ✅ |
| Library capacity | 50k+ | 100k+ ✅ |
| Search latency | < 100ms | ~60ms ✅ |
| Stream startup | < 1s | ~800ms ✅ |

### Optimization Strategies

- **Lazy loading** - Only load visible UI elements
- **Virtual scrolling** - 500 items per page
- **Indexed queries** - FTS5 for instant search
- **Efficient caching** - LRU cache for images
- **Background processing** - Async operations
- **Minimal allocations** - Rust's zero-cost abstractions

---

## 🐛 Bug Reports & Feature Requests

### Reporting Issues

**Found a bug?**
1. Check [existing issues](https://github.com/DrTomLLC/StoryStream/issues)
2. Use [bug report template](.github/ISSUE_TEMPLATE/bug_report.yml)
3. Include:
   - Android version
   - Device model
   - App version
   - Steps to reproduce
   - Logs (Settings → Export Logs)

**Want a feature?**
1. Check [roadmap](docs/ROADMAP.md) first
2. Use [feature request template](.github/ISSUE_TEMPLATE/feature_request.yml)
3. Explain use case and benefit

### Security Issues

**Found a security vulnerability?**
- **DO NOT** open a public issue
- Email: security@storystream.app
- Include detailed report
- Responsible disclosure appreciated

See [SECURITY.md](SECURITY.md) for our security policy.

---

## 💬 Community

### Discussions

Join conversations on [GitHub Discussions](https://github.com/DrTomLLC/StoryStream/discussions):
- 💡 **Ideas** - Feature proposals and brainstorming
- 🙋 **Q&A** - Questions and troubleshooting
- 📢 **Announcements** - Release notes and updates
- 🌟 **Show and Tell** - Share your setup or use cases

### Code of Conduct

StoryStream is committed to providing a welcoming and inclusive environment. All participants must follow our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## 🙏 Acknowledgments

StoryStream builds on the shoulders of giants:

### Projects
- **AntennaPod** - Inspiration for podcast features
- **Voice Audiobook Player** - Inspiration for audiobook management
- **Symphonia** - Rust audio decoding
- **SQLx** - Type-safe database access
- **uniffi** - Seamless Rust ↔ Kotlin bridging

### Content Providers
- **LibriVox** - Free public domain audiobooks
- **Internet Archive** - Massive digital library

### Contributors
See [CONTRIBUTORS.md](CONTRIBUTORS.md) for everyone who has helped build StoryStream.

---

## 📧 Contact

- **General:** hello@storystream.app
- **Licensing:** licensing@storystream.app
- **Security:** security@storystream.app
- **GitHub:** [@DrTomLLC](https://github.com/DrTomLLC)

---

## 📄 Documentation

### User Documentation
- [Installation Guide](docs/INSTALLATION.md)
- [User Manual](docs/USER_GUIDE.md)
- [FAQ](docs/FAQ.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)

### Developer Documentation
- [Architecture Overview](docs/ARCHITECTURE.md)
- [Windows Setup Guide](docs/WINDOWS_SETUP.md)
- [Build Instructions](docs/BUILD_INSTRUCTIONS.md)
- [Testing Guide](docs/TESTING.md)
- [Contributing Guidelines](docs/CONTRIBUTING.md)
- [API Documentation](docs/API.md)
- [Database Schema](docs/DATABASE_SCHEMA.md)
- [Release Process](docs/RELEASE_PROCESS.md)

### Project Management
- [Roadmap](docs/ROADMAP.md)
- [Changelog](CHANGELOG.md)
- [Security Policy](SECURITY.md)

---

## 📈 Project Status

**Current Phase:** Phase 1 (MVP Development)  
**Status:** 🟡 In Active Development  
**Latest Version:** 0.1.0-alpha  
**Contributors:** 1 (seeking more!)  
**Test Coverage:** 75% (target: 80%+)  

### Recent Activity
- ✅ Core playback engine complete
- ✅ Database schema finalized
- ✅ LibriVox integration complete
- ✅ Internet Archive integration complete
- 🔄 UI polish in progress
- 🔄 Comprehensive testing ongoing
- ⏭️ Beta release planned: Month 4

---

## ⭐ Star History

If you find StoryStream useful, please consider starring the project! It helps us gauge interest and attract contributors.

[![Star History Chart](https://api.star-history.com/svg?repos=DrTomLLC/StoryStream&type=Date)](https://star-history.com/#DrTomLLC/StoryStream&Date)

---

## 📜 License Summary

```
StoryStream - Open Source Audiobook & Podcast Player
Copyright (C) 2025 DrTomLLC

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

For commercial licensing options, contact licensing@storystream.app

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Affero General Public License for more details.
```

---

**Built with ❤️ using Rust and Kotlin**

*StoryStream: Your Audio, Your Way* 🎧
