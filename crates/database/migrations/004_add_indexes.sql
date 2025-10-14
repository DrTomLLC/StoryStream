-- Migration 004: Add Performance Indexes
-- Creates indexes for commonly queried columns to improve query performance

-- Books table indexes
CREATE INDEX IF NOT EXISTS idx_books_author ON books(author) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_books_favorite ON books(is_favorite) WHERE deleted_at IS NULL AND is_favorite = 1;
CREATE INDEX IF NOT EXISTS idx_books_last_played ON books(last_played DESC) WHERE deleted_at IS NULL AND last_played IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_books_deleted ON books(deleted_at);
CREATE INDEX IF NOT EXISTS idx_books_series ON books(series, series_position) WHERE deleted_at IS NULL AND series IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_books_added_date ON books(added_date DESC);

-- Chapters table indexes
CREATE INDEX IF NOT EXISTS idx_chapters_book_id ON chapters(book_id);
CREATE INDEX IF NOT EXISTS idx_chapters_book_index ON chapters(book_id, index_number);

-- Bookmarks table indexes
CREATE INDEX IF NOT EXISTS idx_bookmarks_book_id ON bookmarks(book_id);
CREATE INDEX IF NOT EXISTS idx_bookmarks_book_position ON bookmarks(book_id, position_ms);

-- Playback state table indexes
CREATE INDEX IF NOT EXISTS idx_playback_book_id ON playback_state(book_id);

-- Playlist items table indexes
CREATE INDEX IF NOT EXISTS idx_playlist_items_playlist ON playlist_items(playlist_id, position);
CREATE INDEX IF NOT EXISTS idx_playlist_items_book ON playlist_items(book_id);

-- Insert migration version
INSERT OR IGNORE INTO schema_migrations (version) VALUES (4);