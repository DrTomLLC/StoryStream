-- Migration 003: Full-Text Search
-- Creates FTS5 virtual tables for fast text search

-- Drop existing triggers if they exist (to replace with corrected versions)
DROP TRIGGER IF EXISTS books_fts_insert;
DROP TRIGGER IF EXISTS books_fts_update;
DROP TRIGGER IF EXISTS books_fts_delete;
DROP TRIGGER IF EXISTS chapters_fts_insert;
DROP TRIGGER IF EXISTS chapters_fts_update;
DROP TRIGGER IF EXISTS chapters_fts_delete;
DROP TRIGGER IF EXISTS bookmarks_fts_insert;
DROP TRIGGER IF EXISTS bookmarks_fts_update;
DROP TRIGGER IF EXISTS bookmarks_fts_delete;

-- Drop existing FTS tables to recreate without UNINDEXED columns
DROP TABLE IF EXISTS books_fts;
DROP TABLE IF EXISTS chapters_fts;
DROP TABLE IF EXISTS bookmarks_fts;

-- Full-text search for books (no UNINDEXED columns needed)
CREATE VIRTUAL TABLE IF NOT EXISTS books_fts USING fts5(
    title,
    author,
    narrator,
    series,
    description,
    tags,
    content=books,
    content_rowid=rowid
);

-- Triggers to keep FTS table in sync with books table
CREATE TRIGGER books_fts_insert AFTER INSERT ON books BEGIN
    INSERT INTO books_fts(rowid, title, author, narrator, series, description, tags)
    VALUES (NEW.rowid, NEW.title, NEW.author, NEW.narrator, NEW.series, NEW.description, NEW.tags);
END;

CREATE TRIGGER books_fts_update AFTER UPDATE ON books BEGIN
    INSERT INTO books_fts(books_fts, rowid, title, author, narrator, series, description, tags)
    VALUES ('delete', OLD.rowid, OLD.title, OLD.author, OLD.narrator, OLD.series, OLD.description, OLD.tags);
    INSERT INTO books_fts(rowid, title, author, narrator, series, description, tags)
    VALUES (NEW.rowid, NEW.title, NEW.author, NEW.narrator, NEW.series, NEW.description, NEW.tags);
END;

CREATE TRIGGER books_fts_delete AFTER DELETE ON books BEGIN
    INSERT INTO books_fts(books_fts, rowid, title, author, narrator, series, description, tags)
    VALUES ('delete', OLD.rowid, OLD.title, OLD.author, OLD.narrator, OLD.series, OLD.description, OLD.tags);
END;

-- Full-text search for chapters (no UNINDEXED columns needed)
CREATE VIRTUAL TABLE IF NOT EXISTS chapters_fts USING fts5(
    title,
    content=chapters,
    content_rowid=rowid
);

-- Triggers for chapters FTS
CREATE TRIGGER chapters_fts_insert AFTER INSERT ON chapters BEGIN
    INSERT INTO chapters_fts(rowid, title)
    VALUES (NEW.rowid, NEW.title);
END;

CREATE TRIGGER chapters_fts_update AFTER UPDATE ON chapters BEGIN
    INSERT INTO chapters_fts(chapters_fts, rowid, title)
    VALUES ('delete', OLD.rowid, OLD.title);
    INSERT INTO chapters_fts(rowid, title)
    VALUES (NEW.rowid, NEW.title);
END;

CREATE TRIGGER chapters_fts_delete AFTER DELETE ON chapters BEGIN
    INSERT INTO chapters_fts(chapters_fts, rowid, title)
    VALUES ('delete', OLD.rowid, OLD.title);
END;

-- Full-text search for bookmarks (no UNINDEXED columns needed)
CREATE VIRTUAL TABLE IF NOT EXISTS bookmarks_fts USING fts5(
    title,
    note,
    content=bookmarks,
    content_rowid=rowid
);

-- Triggers for bookmarks FTS
CREATE TRIGGER bookmarks_fts_insert AFTER INSERT ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(rowid, title, note)
    VALUES (NEW.rowid, NEW.title, NEW.note);
END;

CREATE TRIGGER bookmarks_fts_update AFTER UPDATE ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, note)
    VALUES ('delete', OLD.rowid, OLD.title, OLD.note);
    INSERT INTO bookmarks_fts(rowid, title, note)
    VALUES (NEW.rowid, NEW.title, NEW.note);
END;

CREATE TRIGGER bookmarks_fts_delete AFTER DELETE ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, note)
    VALUES ('delete', OLD.rowid, OLD.title, OLD.note);
END;

-- Insert migration version
INSERT OR IGNORE INTO schema_migrations (version) VALUES (3);