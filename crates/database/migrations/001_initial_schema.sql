-- Migration 001: Initial Schema
-- Creates all core tables for StoryStream

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_migrations (
                                                 version INTEGER PRIMARY KEY,
                                                 applied_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000)
    );

-- Books table
CREATE TABLE IF NOT EXISTS books (
                                     id TEXT PRIMARY KEY,
                                     title TEXT NOT NULL,
                                     author TEXT,
                                     narrator TEXT,
                                     series TEXT,
                                     series_position REAL,
                                     description TEXT,
                                     language TEXT,
                                     publisher TEXT,
                                     published_date TEXT,
                                     isbn TEXT,
                                     duration_ms INTEGER NOT NULL,
                                     file_path TEXT NOT NULL UNIQUE,
                                     file_size INTEGER NOT NULL,
                                     cover_art_path TEXT,
                                     added_date INTEGER NOT NULL,
                                     last_played INTEGER,
                                     play_count INTEGER NOT NULL DEFAULT 0,
                                     is_favorite INTEGER NOT NULL DEFAULT 0,
                                     rating INTEGER CHECK(rating IS NULL OR (rating >= 1 AND rating <= 5)),
    tags TEXT, -- JSON array
    deleted_at INTEGER
    );

-- Chapters table
CREATE TABLE IF NOT EXISTS chapters (
                                        id TEXT PRIMARY KEY,
                                        book_id TEXT NOT NULL,
                                        title TEXT NOT NULL,
                                        index_number INTEGER NOT NULL,
                                        start_time_ms INTEGER NOT NULL,
                                        end_time_ms INTEGER NOT NULL,
                                        image_path TEXT,
                                        FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
    );

-- Bookmarks table
CREATE TABLE IF NOT EXISTS bookmarks (
                                         id TEXT PRIMARY KEY,
                                         book_id TEXT NOT NULL,
                                         position_ms INTEGER NOT NULL,
                                         title TEXT,
                                         note TEXT,
                                         created_at INTEGER NOT NULL,
                                         updated_at INTEGER NOT NULL,
                                         FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
    );

-- Playback state table
CREATE TABLE IF NOT EXISTS playback_state (
                                              book_id TEXT PRIMARY KEY,
                                              position_ms INTEGER NOT NULL DEFAULT 0,
                                              speed REAL NOT NULL DEFAULT 1.0,
                                              pitch_correction INTEGER NOT NULL DEFAULT 1,
                                              volume INTEGER NOT NULL DEFAULT 100,
                                              is_playing INTEGER NOT NULL DEFAULT 0,
                                              equalizer_preset TEXT, -- JSON
                                              sleep_timer TEXT, -- JSON
                                              skip_silence INTEGER NOT NULL DEFAULT 0,
                                              volume_boost INTEGER NOT NULL DEFAULT 0,
                                              last_updated INTEGER NOT NULL,
                                              FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
    );

-- Playlists table
CREATE TABLE IF NOT EXISTS playlists (
                                         id TEXT PRIMARY KEY,
                                         name TEXT NOT NULL,
                                         description TEXT,
                                         playlist_type TEXT NOT NULL CHECK(playlist_type IN ('Manual', 'Smart')),
    smart_criteria TEXT, -- JSON, required for Smart playlists
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
    );

-- Playlist items table (for manual playlists)
CREATE TABLE IF NOT EXISTS playlist_items (
                                              playlist_id TEXT NOT NULL,
                                              book_id TEXT NOT NULL,
                                              position INTEGER NOT NULL,
                                              added_at INTEGER NOT NULL,
                                              PRIMARY KEY (playlist_id, book_id),
    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
    FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
    );

-- Podcasts table (Phase 2)
CREATE TABLE IF NOT EXISTS podcasts (
                                        id TEXT PRIMARY KEY,
                                        feed_url TEXT NOT NULL UNIQUE,
                                        title TEXT NOT NULL,
                                        description TEXT,
                                        author TEXT,
                                        image_url TEXT,
                                        last_fetched INTEGER,
                                        created_at INTEGER NOT NULL,
                                        updated_at INTEGER NOT NULL
);

-- Podcast episodes table (Phase 2)
CREATE TABLE IF NOT EXISTS podcast_episodes (
                                                id TEXT PRIMARY KEY,
                                                podcast_id TEXT NOT NULL,
                                                title TEXT NOT NULL,
                                                description TEXT,
                                                audio_url TEXT NOT NULL,
                                                duration_ms INTEGER,
                                                published_date INTEGER,
                                                file_path TEXT,
                                                is_downloaded INTEGER NOT NULL DEFAULT 0,
                                                created_at INTEGER NOT NULL,
                                                FOREIGN KEY (podcast_id) REFERENCES podcasts(id) ON DELETE CASCADE
    );

-- Sync changelog table (Phase 3)
CREATE TABLE IF NOT EXISTS sync_changelog (
                                              id INTEGER PRIMARY KEY AUTOINCREMENT,
                                              entity_type TEXT NOT NULL,
                                              entity_id TEXT NOT NULL,
                                              operation TEXT NOT NULL CHECK(operation IN ('INSERT', 'UPDATE', 'DELETE')),
    timestamp INTEGER NOT NULL,
    device_id TEXT NOT NULL,
    synced INTEGER NOT NULL DEFAULT 0
    );

-- Health tracking table
CREATE TABLE IF NOT EXISTS health_log (
                                          id INTEGER PRIMARY KEY AUTOINCREMENT,
                                          event_type TEXT NOT NULL,
                                          severity TEXT NOT NULL CHECK(severity IN ('Info', 'Warning', 'Error', 'Critical')),
    message TEXT NOT NULL,
    details TEXT, -- JSON
    timestamp INTEGER NOT NULL
    );

-- Backup tracking table
CREATE TABLE IF NOT EXISTS backups (
                                       id INTEGER PRIMARY KEY AUTOINCREMENT,
                                       backup_type TEXT NOT NULL CHECK(backup_type IN ('Full', 'Incremental')),
    file_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    verified INTEGER NOT NULL DEFAULT 0
    );

-- Insert initial migration version
INSERT OR IGNORE INTO schema_migrations (version) VALUES (1);