// FILE: crates/media-engine/src/chapters.rs
//! Chapter navigation and management

use std::time::Duration;

/// Represents a chapter marker in an audiobook
#[derive(Debug, Clone, PartialEq)]
pub struct ChapterMarker {
    /// Chapter index (0-based)
    pub index: usize,
    /// Chapter title
    pub title: String,
    /// Start time in seconds
    pub start_time: f64,
    /// End time in seconds (exclusive)
    pub end_time: f64,
}

impl ChapterMarker {
    /// Creates a new chapter marker
    pub fn new(index: usize, title: String, start_time: f64, end_time: f64) -> Self {
        Self {
            index,
            title,
            start_time,
            end_time,
        }
    }

    /// Returns the chapter duration in seconds
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }

    /// Checks if a given position falls within this chapter
    pub fn contains(&self, position: f64) -> bool {
        position >= self.start_time && position < self.end_time
    }

    /// Returns duration as std::time::Duration
    pub fn duration_std(&self) -> Duration {
        Duration::from_secs_f64(self.duration())
    }
}

/// Manages chapters for an audiobook
#[derive(Debug, Clone)]
pub struct ChapterList {
    chapters: Vec<ChapterMarker>,
    current_index: Option<usize>,
}

impl ChapterList {
    /// Creates a new chapter list with no chapters
    pub fn new() -> Self {
        Self {
            chapters: Vec::new(),
            current_index: None,
        }
    }

    /// Creates a chapter list with the given chapters
    pub fn with_chapters(chapters: Vec<ChapterMarker>) -> Self {
        let current_index = if chapters.is_empty() { None } else { Some(0) };
        Self {
            chapters,
            current_index,
        }
    }

    /// Adds a chapter
    pub fn add_chapter(&mut self, chapter: ChapterMarker) {
        self.chapters.push(chapter);
        self.chapters.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

        if self.current_index.is_none() && !self.chapters.is_empty() {
            self.current_index = Some(0);
        }
    }

    /// Returns the total number of chapters
    pub fn chapter_count(&self) -> usize {
        self.chapters.len()
    }

    /// Returns true if there are chapters
    pub fn has_chapters(&self) -> bool {
        !self.chapters.is_empty()
    }

    /// Gets a chapter by index
    pub fn get_chapter(&self, index: usize) -> Option<&ChapterMarker> {
        self.chapters.get(index)
    }

    /// Gets the current chapter
    pub fn current_chapter(&self) -> Option<&ChapterMarker> {
        self.current_index.and_then(|idx| self.chapters.get(idx))
    }

    /// Gets the current chapter index
    pub fn current_chapter_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Finds the chapter that contains the given position
    pub fn chapter_at_position(&self, position: f64) -> Option<&ChapterMarker> {
        self.chapters.iter().find(|ch| ch.contains(position))
    }

    /// Updates current chapter based on position
    pub fn update_position(&mut self, position: f64) {
        if let Some(idx) = self.chapters.iter().position(|ch| ch.contains(position)) {
            self.current_index = Some(idx);
        }
    }

    /// Gets the next chapter, if any
    pub fn next_chapter(&self) -> Option<&ChapterMarker> {
        self.current_index
            .and_then(|idx| self.chapters.get(idx + 1))
    }

    /// Gets the previous chapter, if any
    pub fn previous_chapter(&self) -> Option<&ChapterMarker> {
        self.current_index
            .and_then(|idx| if idx > 0 { self.chapters.get(idx - 1) } else { None })
    }

    /// Moves to the next chapter, returning its start time
    pub fn go_to_next(&mut self) -> Option<f64> {
        if let Some(current) = self.current_index {
            if current + 1 < self.chapters.len() {
                self.current_index = Some(current + 1);
                return self.chapters.get(current + 1).map(|ch| ch.start_time);
            }
        }
        None
    }

    /// Moves to the previous chapter, returning its start time
    pub fn go_to_previous(&mut self) -> Option<f64> {
        if let Some(current) = self.current_index {
            if current > 0 {
                self.current_index = Some(current - 1);
                return self.chapters.get(current - 1).map(|ch| ch.start_time);
            }
        }
        None
    }

    /// Jumps to a specific chapter by index, returning its start time
    pub fn go_to_chapter(&mut self, index: usize) -> Option<f64> {
        if index < self.chapters.len() {
            self.current_index = Some(index);
            self.chapters.get(index).map(|ch| ch.start_time)
        } else {
            None
        }
    }

    /// Returns all chapters
    pub fn chapters(&self) -> &[ChapterMarker] {
        &self.chapters
    }

    /// Returns formatted chapter info (e.g., "3/15")
    pub fn chapter_progress(&self) -> String {
        match (self.current_index, self.chapter_count()) {
            (Some(idx), count) if count > 0 => format!("{}/{}", idx + 1, count),
            (None, count) if count > 0 => format!("?/{}", count),
            _ => "No chapters".to_string(),
        }
    }
}

impl Default for ChapterList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_chapters() -> Vec<ChapterMarker> {
        vec![
            ChapterMarker::new(0, "Introduction".to_string(), 0.0, 300.0),
            ChapterMarker::new(1, "Chapter 1".to_string(), 300.0, 900.0),
            ChapterMarker::new(2, "Chapter 2".to_string(), 900.0, 1500.0),
            ChapterMarker::new(3, "Epilogue".to_string(), 1500.0, 1800.0),
        ]
    }

    #[test]
    fn test_chapter_creation() {
        let ch = ChapterMarker::new(0, "Test".to_string(), 0.0, 100.0);
        assert_eq!(ch.index, 0);
        assert_eq!(ch.title, "Test");
        assert_eq!(ch.duration(), 100.0);
    }

    #[test]
    fn test_chapter_contains() {
        let ch = ChapterMarker::new(0, "Test".to_string(), 100.0, 200.0);
        assert!(ch.contains(100.0));
        assert!(ch.contains(150.0));
        assert!(ch.contains(199.9));
        assert!(!ch.contains(200.0));
        assert!(!ch.contains(99.9));
    }

    #[test]
    fn test_chapter_list_new() {
        let list = ChapterList::new();
        assert_eq!(list.chapter_count(), 0);
        assert!(!list.has_chapters());
        assert!(list.current_chapter().is_none());
    }

    #[test]
    fn test_chapter_list_with_chapters() {
        let chapters = create_test_chapters();
        let list = ChapterList::with_chapters(chapters);

        assert_eq!(list.chapter_count(), 4);
        assert!(list.has_chapters());
        assert_eq!(list.current_chapter_index(), Some(0));
    }

    #[test]
    fn test_add_chapter() {
        let mut list = ChapterList::new();
        list.add_chapter(ChapterMarker::new(0, "Ch1".to_string(), 0.0, 100.0));

        assert_eq!(list.chapter_count(), 1);
        assert!(list.has_chapters());
    }

    #[test]
    fn test_chapter_at_position() {
        let chapters = create_test_chapters();
        let list = ChapterList::with_chapters(chapters);

        assert_eq!(list.chapter_at_position(50.0).map(|ch| ch.index), Some(0));
        assert_eq!(list.chapter_at_position(500.0).map(|ch| ch.index), Some(1));
        assert_eq!(list.chapter_at_position(1200.0).map(|ch| ch.index), Some(2));
    }

    #[test]
    fn test_update_position() {
        let chapters = create_test_chapters();
        let mut list = ChapterList::with_chapters(chapters);

        list.update_position(500.0);
        assert_eq!(list.current_chapter_index(), Some(1));

        list.update_position(1200.0);
        assert_eq!(list.current_chapter_index(), Some(2));
    }

    #[test]
    fn test_next_chapter() {
        let chapters = create_test_chapters();
        let list = ChapterList::with_chapters(chapters);

        assert_eq!(list.next_chapter().map(|ch| ch.index), Some(1));
    }

    #[test]
    fn test_previous_chapter() {
        let chapters = create_test_chapters();
        let mut list = ChapterList::with_chapters(chapters);

        list.go_to_chapter(2);
        assert_eq!(list.previous_chapter().map(|ch| ch.index), Some(1));
    }

    #[test]
    fn test_go_to_next() {
        let chapters = create_test_chapters();
        let mut list = ChapterList::with_chapters(chapters);

        let start_time = list.go_to_next();
        assert_eq!(start_time, Some(300.0));
        assert_eq!(list.current_chapter_index(), Some(1));
    }

    #[test]
    fn test_go_to_previous() {
        let chapters = create_test_chapters();
        let mut list = ChapterList::with_chapters(chapters);

        list.go_to_chapter(2);
        let start_time = list.go_to_previous();
        assert_eq!(start_time, Some(300.0));
        assert_eq!(list.current_chapter_index(), Some(1));
    }

    #[test]
    fn test_go_to_chapter() {
        let chapters = create_test_chapters();
        let mut list = ChapterList::with_chapters(chapters);

        let start_time = list.go_to_chapter(3);
        assert_eq!(start_time, Some(1500.0));
        assert_eq!(list.current_chapter_index(), Some(3));
    }

    #[test]
    fn test_chapter_progress() {
        let chapters = create_test_chapters();
        let mut list = ChapterList::with_chapters(chapters);

        assert_eq!(list.chapter_progress(), "1/4");

        list.go_to_next();
        assert_eq!(list.chapter_progress(), "2/4");
    }

    #[test]
    fn test_no_next_at_end() {
        let chapters = create_test_chapters();
        let mut list = ChapterList::with_chapters(chapters);

        list.go_to_chapter(3); // Last chapter
        assert!(list.go_to_next().is_none());
    }

    #[test]
    fn test_no_previous_at_start() {
        let chapters = create_test_chapters();
        let mut list = ChapterList::with_chapters(chapters);

        assert!(list.go_to_previous().is_none());
    }
}