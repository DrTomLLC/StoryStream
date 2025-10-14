//! Database query operations organized by entity

pub mod books;
pub mod bookmarks;
pub mod chapters;
pub mod playback;
pub mod playlists;

// Re-export commonly used query functions
pub use books::{
    create_book, delete_book, get_book, get_books_by_author, get_favorite_books,
    get_recently_played_books, list_books, update_book,
};
pub use bookmarks::{create_bookmark, delete_bookmark, get_book_bookmarks, get_bookmark};
pub use chapters::{create_chapter, delete_chapter, get_book_chapters, get_chapter};
pub use playback::{create_playback_state, get_playback_state, update_playback_state};
pub use playlists::{
    add_book_to_playlist, create_playlist, delete_playlist, get_playlist, get_playlist_books,
    remove_book_from_playlist,
};