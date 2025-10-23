// crates/android-bridge/src/library_bridge.rs
//! Library bridge for exposing library management functionality to Android
//!
//! This module provides JNI bindings for:
//! - Library scanning and import
//! - Book metadata access
//! - Library queries and searches
//! - Collection management

use crate::ffi::{
    bool_to_jboolean, jstring_to_string, option_string_to_jstring, string_to_jstring, FfiError,
    FfiResult, HandleManager,
};
use crate::jni_safe;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jstring};
use jni::JNIEnv;
use once_cell::sync::Lazy;
use std::sync::Arc;

// Placeholder types until we implement proper integration
pub struct LibraryManager {
    // Internal state will be added when integrating with library module
}

pub struct BookMetadata {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub narrator: Option<String>,
    pub duration: Option<f64>,
    pub file_path: String,
    pub cover_art_path: Option<String>,
}

pub struct ScanProgress {
    pub files_scanned: usize,
    pub books_found: usize,
    pub is_complete: bool,
}

// Global handle manager for library instances
static LIBRARY_HANDLES: Lazy<HandleManager<Arc<LibraryManager>>> = Lazy::new(HandleManager::new);

/// Creates a new library manager instance
///
/// # Safety
/// The returned handle must be properly released using `library_destroy`
/// to prevent memory leaks.
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    library_root_path: JString,
) -> jlong {
    jni_safe!(env, 0, {
        let path = jstring_to_string(&mut env, library_root_path)?;

        // TODO: Create actual LibraryManager from library module
        let library = Arc::new(LibraryManager {});

        let handle = LIBRARY_HANDLES.insert(library);
        crate::ffi::log_info(
            "StoryStream",
            &format!("Created library handle: {} for path: {}", handle, path),
        );

        Ok(handle)
    })
}

/// Destroys a library manager instance
///
/// # Safety
/// After calling this function, the handle becomes invalid and must not be used.
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeDestroy(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    jni_safe!(env, (), {
        LIBRARY_HANDLES.remove(handle)?;
        crate::ffi::log_info("StoryStream", &format!("Destroyed library handle: {}", handle));
        Ok(())
    })
}

/// Starts scanning the library for audiobook files
///
/// This is an asynchronous operation. Use `library_get_scan_progress` to check status.
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeStartScan(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    jni_safe!(env, bool_to_jboolean(false), {
        LIBRARY_HANDLES.get(handle)?;

        // TODO: Start actual scan using library module
        crate::ffi::log_info("StoryStream", "Library scan started");

        Ok(bool_to_jboolean(true))
    })
}

/// Cancels an ongoing library scan
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeCancelScan(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    jni_safe!(env, bool_to_jboolean(false), {
        LIBRARY_HANDLES.get(handle)?;

        // TODO: Cancel scan using library module
        crate::ffi::log_info("StoryStream", "Library scan cancelled");

        Ok(bool_to_jboolean(true))
    })
}

/// Checks if a scan is currently in progress
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeIsScanInProgress(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    jni_safe!(env, bool_to_jboolean(false), {
        LIBRARY_HANDLES.get(handle)?;

        // TODO: Check scan status from library module
        Ok(bool_to_jboolean(false))
    })
}

/// Gets the current scan progress
///
/// Returns the number of files scanned so far, or -1 if no scan is in progress.
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeGetFilesScanned(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    jni_safe!(env, -1, {
        LIBRARY_HANDLES.get(handle)?;

        // TODO: Get scan progress from library module
        Ok(0)
    })
}

/// Gets the number of books found during scanning
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeGetBooksFound(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    jni_safe!(env, 0, {
        LIBRARY_HANDLES.get(handle)?;

        // TODO: Get books found count from library module
        Ok(0)
    })
}

/// Gets the total number of books in the library
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeGetBookCount(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    jni_safe!(env, 0, {
        LIBRARY_HANDLES.get(handle)?;

        // TODO: Get total book count from library module
        Ok(0)
    })
}

/// Imports a specific audiobook file into the library
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeImportFile(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    file_path: JString,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let path = jstring_to_string(&mut env, file_path)?;

        // TODO: Import file using library module
        crate::ffi::log_info("StoryStream", &format!("Importing file: {}", path));

        // Return a placeholder book ID
        string_to_jstring(&mut env, "placeholder-book-id")
    })
}

/// Gets book metadata by ID
///
/// Returns null if the book doesn't exist.
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeGetBookTitle(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: JString,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_to_string(&mut env, book_id)?;

        // TODO: Get book title from library module
        crate::ffi::log_debug("StoryStream", &format!("Get title for book: {}", id));

        string_to_jstring(&mut env, "Unknown Title")
    })
}

/// Gets the author name for a book
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeGetBookAuthor(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: JString,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_to_string(&mut env, book_id)?;

        // TODO: Get book author from library module
        crate::ffi::log_debug("StoryStream", &format!("Get author for book: {}", id));

        option_string_to_jstring(&mut env, Some("Unknown Author"))
    })
}

/// Gets the narrator name for a book
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeGetBookNarrator(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: JString,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_to_string(&mut env, book_id)?;

        // TODO: Get book narrator from library module
        crate::ffi::log_debug("StoryStream", &format!("Get narrator for book: {}", id));

        Ok(std::ptr::null_mut()) // Return null for no narrator
    })
}

/// Gets the file path for a book
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeGetBookFilePath(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: JString,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_to_string(&mut env, book_id)?;

        // TODO: Get book file path from library module
        crate::ffi::log_debug("StoryStream", &format!("Get file path for book: {}", id));

        string_to_jstring(&mut env, "/path/to/audiobook.mp3")
    })
}

/// Gets the cover art path for a book
///
/// Returns null if no cover art is available.
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeGetBookCoverPath(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: JString,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_to_string(&mut env, book_id)?;

        // TODO: Get cover art path from library module
        crate::ffi::log_debug("StoryStream", &format!("Get cover art for book: {}", id));

        Ok(std::ptr::null_mut()) // Return null for no cover art
    })
}

/// Searches the library for books matching a query
///
/// Returns an array of book IDs as a JSON string.
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeSearch(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    query: JString,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let query_str = jstring_to_string(&mut env, query)?;

        // TODO: Perform actual search using library module
        crate::ffi::log_debug("StoryStream", &format!("Search query: {}", query_str));

        // Return empty JSON array for now
        string_to_jstring(&mut env, "[]")
    })
}

/// Deletes a book from the library
///
/// Note: This removes the database entry but does NOT delete the physical file.
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeDeleteBook(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: JString,
) -> jboolean {
    jni_safe!(env, bool_to_jboolean(false), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_to_string(&mut env, book_id)?;

        // TODO: Delete book using library module
        crate::ffi::log_info("StoryStream", &format!("Delete book: {}", id));

        Ok(bool_to_jboolean(true))
    })
}

/// Gets a list of all book IDs in the library as a JSON array string
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeListAllBooks(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;

        // TODO: Get all book IDs from library module
        crate::ffi::log_debug("StoryStream", "List all books");

        // Return empty JSON array for now
        string_to_jstring(&mut env, "[]")
    })
}

/// Gets recently added books (returns JSON array of book IDs)
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeGetRecentBooks(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    limit: jint,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;

        if limit < 0 {
            return Err(FfiError::Core("Limit must be non-negative".to_string()));
        }

        // TODO: Get recent books from library module
        crate::ffi::log_debug("StoryStream", &format!("Get {} recent books", limit));

        // Return empty JSON array for now
        string_to_jstring(&mut env, "[]")
    })
}

/// Refreshes metadata for a specific book
#[no_mangle]
pub extern "C" fn Java_com_storystream_Library_nativeRefreshMetadata(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: JString,
) -> jboolean {
    jni_safe!(env, bool_to_jboolean(false), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_to_string(&mut env, book_id)?;

        // TODO: Refresh metadata using library module
        crate::ffi::log_info("StoryStream", &format!("Refresh metadata for: {}", id));

        Ok(bool_to_jboolean(true))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_handles_lifecycle() {
        let library1 = Arc::new(LibraryManager {});
        let library2 = Arc::new(LibraryManager {});

        let handle1 = LIBRARY_HANDLES.insert(library1);
        let handle2 = LIBRARY_HANDLES.insert(library2);

        assert_ne!(handle1, handle2);
        assert!(LIBRARY_HANDLES.contains(handle1));
        assert!(LIBRARY_HANDLES.contains(handle2));

        LIBRARY_HANDLES.remove(handle1).unwrap();
        assert!(!LIBRARY_HANDLES.contains(handle1));

        LIBRARY_HANDLES.remove(handle2).unwrap();
    }

    #[test]
    fn test_invalid_library_handle() {
        let result = LIBRARY_HANDLES.get(88888);
        assert!(result.is_err());
    }
}