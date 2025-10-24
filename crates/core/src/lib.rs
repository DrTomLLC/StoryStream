pub mod error;
pub mod types;

// Re-export commonly used types
pub use error::{AppError, ErrorSeverity, RecoveryAction, Result};
pub use types::{
    AudioFormat, AudioMetadata, Book, BookId, Bookmark, BookmarkId, Chapter, ChapterId, Duration,
    LibraryStats, PlaybackSpeed, PlaybackState, PlaybackStats, Playlist, PlaylistId, PlaylistItem,
    PlaylistType, Timestamp,
};
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
