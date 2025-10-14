-- Migration 002: Add Indexes for Performance
-- Creates indexes on frequently queried columns

-- Books indexes
CREATE INDEX IF NOT EXISTS idx_books_author ON books(author) WHERE author IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_books_narrator ON books(narrator) WHERE narrator IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_books_series ON books(series) WHERE series IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_books_added_date ON books(added_date DESC);
CREATE INDEX IF NOT EXISTS idx_books_last_played ON books(last_played DESC) WHERE last_played IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_books_is_favorite ON books(is_favorite) WHERE is_favorite = 1;
CREATE INDEX IF NOT EXISTS idx_books_rating ON books(rating DESC) WHERE rating IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_books_deleted_at ON books(deleted_at) WHERE deleted_at IS NOT NULL;

-- Chapters indexes
CREATE INDEX IF NOT EXISTS idx_chapters_book_id ON chapters(book_id);
CREATE INDEX IF NOT EXISTS idx_chapters_book_index ON chapters(book_id, index_number);

-- Bookmarks indexes
CREATE INDEX IF NOT EXISTS idx_bookmarks_book_id ON bookmarks(book_id);
CREATE INDEX IF NOT EXISTS idx_bookmarks_created_at ON bookmarks(created_at DESC);

-- Playlist items indexes
CREATE INDEX IF NOT EXISTS idx_playlist_items_playlist_id ON playlist_items(playlist_id, position);
CREATE INDEX IF NOT EXISTS idx_playlist_items_book_id ON playlist_items(book_id);

-- Podcasts indexes (Phase 2)
CREATE INDEX IF NOT EXISTS idx_podcast_episodes_podcast_id ON podcast_episodes(podcast_id);
CREATE INDEX IF NOT EXISTS idx_podcast_episodes_published_date ON podcast_episodes(published_date DESC);

-- Sync changelog indexes (Phase 3)
CREATE INDEX IF NOT EXISTS idx_sync_changelog_synced ON sync_changelog(synced) WHERE synced = 0;
CREATE INDEX IF NOT EXISTS idx_sync_changelog_entity ON sync_changelog(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_sync_changelog_timestamp ON sync_changelog(timestamp DESC);

-- Health log indexes
CREATE INDEX IF NOT EXISTS idx_health_log_severity ON health_log(severity);
CREATE INDEX IF NOT EXISTS idx_health_log_timestamp ON health_log(timestamp DESC);

-- Backups indexes
CREATE INDEX IF NOT EXISTS idx_backups_created_at ON backups(created_at DESC);

-- Insert migration version
INSERT OR IGNORE INTO schema_migrations (version) VALUES (2);