//! Library and playback statistics

use crate::types::Duration;
use serde::{Deserialize, Serialize};

/// Library-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStats {
    pub total_books: usize,
    pub total_chapters: usize,
    pub total_bookmarks: usize,
    pub total_playlists: usize,
    pub total_duration: Duration,
    pub total_size_bytes: u64,
    pub favorite_count: usize,
    pub unfinished_count: usize,
    pub finished_count: usize,
    pub authors_count: usize,
    pub narrators_count: usize,
    pub series_count: usize,
}

impl LibraryStats {
    /// Creates empty statistics
    pub fn empty() -> Self {
        Self {
            total_books: 0,
            total_chapters: 0,
            total_bookmarks: 0,
            total_playlists: 0,
            total_duration: Duration::from_millis(0),
            total_size_bytes: 0,
            favorite_count: 0,
            unfinished_count: 0,
            finished_count: 0,
            authors_count: 0,
            narrators_count: 0,
            series_count: 0,
        }
    }

    /// Returns the average book duration
    pub fn average_book_duration(&self) -> Duration {
        if self.total_books == 0 {
            return Duration::from_millis(0);
        }
        Duration::from_millis(self.total_duration.as_millis() / self.total_books as u64)
    }

    /// Returns the average book size in bytes
    pub fn average_book_size(&self) -> u64 {
        if self.total_books == 0 {
            return 0;
        }
        self.total_size_bytes / self.total_books as u64
    }

    /// Returns the total size in megabytes
    pub fn total_size_mb(&self) -> u64 {
        self.total_size_bytes / 1_000_000
    }

    /// Returns the total size in gigabytes
    pub fn total_size_gb(&self) -> f64 {
        self.total_size_bytes as f64 / 1_000_000_000.0
    }

    /// Returns the percentage of favorite books
    pub fn favorite_percentage(&self) -> f64 {
        if self.total_books == 0 {
            return 0.0;
        }
        (self.favorite_count as f64 / self.total_books as f64) * 100.0
    }

    /// Returns the percentage of finished books
    pub fn finished_percentage(&self) -> f64 {
        if self.total_books == 0 {
            return 0.0;
        }
        (self.finished_count as f64 / self.total_books as f64) * 100.0
    }

    /// Returns the average bookmarks per book
    pub fn average_bookmarks_per_book(&self) -> f64 {
        if self.total_books == 0 {
            return 0.0;
        }
        self.total_bookmarks as f64 / self.total_books as f64
    }

    /// Returns the average chapters per book
    pub fn average_chapters_per_book(&self) -> f64 {
        if self.total_books == 0 {
            return 0.0;
        }
        self.total_chapters as f64 / self.total_books as f64
    }
}

impl Default for LibraryStats {
    fn default() -> Self {
        Self::empty()
    }
}

/// Playback statistics for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackStats {
    pub total_listening_time: Duration,
    pub books_started: usize,
    pub books_finished: usize,
    pub total_sessions: usize,
    pub average_session_duration: Duration,
    pub favorite_authors: Vec<String>,
    pub favorite_narrators: Vec<String>,
    pub most_played_book_id: Option<String>,
}

impl PlaybackStats {
    /// Creates empty playback statistics
    pub fn empty() -> Self {
        Self {
            total_listening_time: Duration::from_millis(0),
            books_started: 0,
            books_finished: 0,
            total_sessions: 0,
            average_session_duration: Duration::from_millis(0),
            favorite_authors: Vec::new(),
            favorite_narrators: Vec::new(),
            most_played_book_id: None,
        }
    }

    /// Returns the total listening time in hours
    pub fn total_hours(&self) -> f64 {
        self.total_listening_time.as_seconds() as f64 / 3600.0
    }

    /// Returns the completion rate (finished / started)
    pub fn completion_rate(&self) -> f64 {
        if self.books_started == 0 {
            return 0.0;
        }
        (self.books_finished as f64 / self.books_started as f64) * 100.0
    }

    /// Returns true if the user has any playback history
    pub fn has_history(&self) -> bool {
        self.total_sessions > 0 || self.books_started > 0
    }
}

impl Default for PlaybackStats {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_stats_empty() {
        let stats = LibraryStats::empty();
        assert_eq!(stats.total_books, 0);
        assert_eq!(stats.total_duration.as_millis(), 0);
        assert_eq!(stats.total_size_bytes, 0);
    }

    #[test]
    fn test_library_stats_default() {
        let stats = LibraryStats::default();
        assert_eq!(stats.total_books, 0);
    }

    #[test]
    fn test_library_stats_average_book_duration() {
        let mut stats = LibraryStats::empty();
        stats.total_books = 10;
        stats.total_duration = Duration::from_seconds(36000); // 10 hours

        let avg = stats.average_book_duration();
        assert_eq!(avg.as_seconds(), 3600); // 1 hour per book
    }

    #[test]
    fn test_library_stats_average_book_duration_zero_books() {
        let stats = LibraryStats::empty();
        let avg = stats.average_book_duration();
        assert_eq!(avg.as_millis(), 0);
    }

    #[test]
    fn test_library_stats_average_book_size() {
        let mut stats = LibraryStats::empty();
        stats.total_books = 5;
        stats.total_size_bytes = 50_000_000; // 50 MB

        assert_eq!(stats.average_book_size(), 10_000_000); // 10 MB per book
    }

    #[test]
    fn test_library_stats_average_book_size_zero_books() {
        let stats = LibraryStats::empty();
        assert_eq!(stats.average_book_size(), 0);
    }

    #[test]
    fn test_library_stats_total_size_mb() {
        let mut stats = LibraryStats::empty();
        stats.total_size_bytes = 50_000_000;
        assert_eq!(stats.total_size_mb(), 50);
    }

    #[test]
    fn test_library_stats_total_size_gb() {
        let mut stats = LibraryStats::empty();
        stats.total_size_bytes = 5_000_000_000;
        assert!((stats.total_size_gb() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_library_stats_favorite_percentage() {
        let mut stats = LibraryStats::empty();
        stats.total_books = 100;
        stats.favorite_count = 25;

        assert!((stats.favorite_percentage() - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_library_stats_favorite_percentage_zero_books() {
        let stats = LibraryStats::empty();
        assert_eq!(stats.favorite_percentage(), 0.0);
    }

    #[test]
    fn test_library_stats_finished_percentage() {
        let mut stats = LibraryStats::empty();
        stats.total_books = 50;
        stats.finished_count = 10;

        assert!((stats.finished_percentage() - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_library_stats_average_bookmarks_per_book() {
        let mut stats = LibraryStats::empty();
        stats.total_books = 10;
        stats.total_bookmarks = 50;

        assert!((stats.average_bookmarks_per_book() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_library_stats_average_chapters_per_book() {
        let mut stats = LibraryStats::empty();
        stats.total_books = 20;
        stats.total_chapters = 400;

        assert!((stats.average_chapters_per_book() - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_playback_stats_empty() {
        let stats = PlaybackStats::empty();
        assert_eq!(stats.total_listening_time.as_millis(), 0);
        assert_eq!(stats.books_started, 0);
        assert_eq!(stats.books_finished, 0);
        assert!(!stats.has_history());
    }

    #[test]
    fn test_playback_stats_default() {
        let stats = PlaybackStats::default();
        assert_eq!(stats.total_sessions, 0);
    }

    #[test]
    fn test_playback_stats_total_hours() {
        let mut stats = PlaybackStats::empty();
        stats.total_listening_time = Duration::from_seconds(7200); // 2 hours

        assert!((stats.total_hours() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_playback_stats_completion_rate() {
        let mut stats = PlaybackStats::empty();
        stats.books_started = 100;
        stats.books_finished = 75;

        assert!((stats.completion_rate() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_playback_stats_completion_rate_zero_started() {
        let stats = PlaybackStats::empty();
        assert_eq!(stats.completion_rate(), 0.0);
    }

    #[test]
    fn test_playback_stats_has_history() {
        let mut stats = PlaybackStats::empty();
        assert!(!stats.has_history());

        stats.total_sessions = 1;
        assert!(stats.has_history());

        let mut stats2 = PlaybackStats::empty();
        stats2.books_started = 1;
        assert!(stats2.has_history());
    }

    #[test]
    fn test_playback_stats_favorite_authors() {
        let mut stats = PlaybackStats::empty();
        stats.favorite_authors = vec!["Author 1".to_string(), "Author 2".to_string()];

        assert_eq!(stats.favorite_authors.len(), 2);
        assert!(stats.favorite_authors.contains(&"Author 1".to_string()));
    }

    #[test]
    fn test_playback_stats_favorite_narrators() {
        let mut stats = PlaybackStats::empty();
        stats.favorite_narrators = vec!["Narrator 1".to_string()];

        assert_eq!(stats.favorite_narrators.len(), 1);
    }

    #[test]
    fn test_playback_stats_most_played_book() {
        let mut stats = PlaybackStats::empty();
        assert!(stats.most_played_book_id.is_none());

        stats.most_played_book_id = Some("book-123".to_string());
        assert_eq!(stats.most_played_book_id, Some("book-123".to_string()));
    }
}
