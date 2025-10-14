//! Domain types for StoryStream
//!
//! This module contains all domain models organized by responsibility:
//! - `book`: Book and Chapter types
//! - `playback`: Playback state and audio settings
//! - `bookmark`: User bookmarks
//! - `playlist`: Playlists and playlist items
//! - `metadata`: Audio format detection and metadata
//! - `stats`: Library statistics
//! - `common`: Shared traits and utilities

mod book;
mod bookmark;
mod common;
mod metadata;
mod playback;
mod playlist;
mod stats;

// Re-export all public types
pub use book::{Book, BookId, Chapter, ChapterId};
pub use bookmark::{Bookmark, BookmarkId};
pub use common::{Duration, Timestamp, Validator};
pub use metadata::{AudioFormat, AudioMetadata, CoverArt};
pub use playback::{
    EqualizerBand, EqualizerPreset, PlaybackSpeed, PlaybackState, SleepTimer, SleepTimerState,
};
pub use playlist::{Playlist, PlaylistId, PlaylistItem, PlaylistType, SmartPlaylistCriteria};
pub use stats::{LibraryStats, PlaybackStats};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_types_are_exported() {
        // Ensure all types compile and are accessible
        let _book_id: BookId = BookId::new();
        let _chapter_id: ChapterId = ChapterId::new();
        let _bookmark_id: BookmarkId = BookmarkId::new();
        let _playlist_id: PlaylistId = PlaylistId::new();
    }

    #[test]
    fn test_timestamp_ordering() {
        let t1 = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = Timestamp::now();
        assert!(t2 > t1);
    }

    #[test]
    fn test_duration_formatting() {
        let d = Duration::from_seconds(3665); // 1h 1m 5s
        assert!(d.to_string().contains("1:01:05"));
    }
}