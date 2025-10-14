-- Migration 002: Playlists
-- Playlists tables were already included in 001_initial_schema.sql
-- This migration exists for version tracking purposes only

INSERT OR IGNORE INTO schema_migrations (version) VALUES (2);