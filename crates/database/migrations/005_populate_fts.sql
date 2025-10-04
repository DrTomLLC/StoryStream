-- Migration 005: Populate FTS Tables
-- Populates the full-text search tables with existing data from the main tables

-- Populate books FTS table with existing books
INSERT INTO books_fts(rowid, title, author, narrator, series, description, tags)
SELECT rowid, title, author, narrator, series, description, tags
FROM books;

-- Populate chapters FTS table with existing chapters
INSERT INTO chapters_fts(rowid, title)
SELECT rowid, title
FROM chapters;

-- Populate bookmarks FTS table with existing bookmarks
INSERT INTO bookmarks_fts(rowid, title, note)
SELECT rowid, title, note
FROM bookmarks;

-- Insert migration version
INSERT OR IGNORE INTO schema_migrations (version) VALUES (5);