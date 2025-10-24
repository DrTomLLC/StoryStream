// Library management bridge for Android JNI
//
// This module provides JNI bindings for audiobook library management including
// initialization, scanning, and metadata retrieval.

use crate::ffi::{
    bool_to_jboolean, jstring_raw_to_string, option_string_to_jstring, string_to_jstring, FfiError,
    FfiResult, HandleManager,
};
use jni::{objects::JClass, sys::{jboolean, jint, jlong, jstring}, JNIEnv};
use std::panic;
use crate::jni_safe;
// Required for jni_safe! macro

/// Global library handle manager
static LIBRARY_HANDLES: once_cell::sync::Lazy<HandleManager<LibraryContext>> =
    once_cell::sync::Lazy::new(HandleManager::default);

/// Library context holding state for a library instance
#[derive(Clone)]
struct LibraryContext {
    root_path: String,
    initialized: bool,
}

impl LibraryContext {
    fn new(root_path: String) -> Self {
        Self {
            root_path,
            initialized: true,
        }
    }
}

/// Initialize a new library instance
///
/// # Safety
/// Must be called from Java with valid JNI environment and parameters
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeInitialize(
    mut env: JNIEnv,
    _class: JClass,
    library_root_path: jstring,
) -> jlong {
    jni_safe!(env, 0, {
        let path = jstring_raw_to_string(&mut env, library_root_path)?;

        // Validate path
        if path.is_empty() {
            return Err(FfiError::General("Library root path cannot be empty".to_string()));
        }

        let context = LibraryContext::new(path.clone());
        let handle = LIBRARY_HANDLES.insert(context);

        crate::ffi::log_info("StoryStream", &format!("Initialized library at: {} (handle: {})", path, handle));

        Ok(handle)
    })
}

/// Destroy a library instance and free resources
///
/// # Safety
/// Must be called from Java with valid handle
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeDestroy(
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

/// Scan library for audiobooks
///
/// # Safety
/// Must be called from Java with valid handle
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeScanLibrary(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    jni_safe!(env, bool_to_jboolean(false), {
        LIBRARY_HANDLES.get(handle)?;

        // Placeholder: In real implementation, would scan filesystem
        crate::ffi::log_info("StoryStream", "Scanning library for audiobooks");

        Ok(bool_to_jboolean(true))
    })
}

/// Check if library is initialized
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeIsInitialized(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    jni_safe!(env, bool_to_jboolean(false), {
        LIBRARY_HANDLES.get(handle)?;

        // Placeholder: Always return true for valid handles
        Ok(bool_to_jboolean(true))
    })
}

/// Check if library directory exists
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeDirectoryExists(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    jni_safe!(env, bool_to_jboolean(false), {
        LIBRARY_HANDLES.get(handle)?;

        // Placeholder: Would check actual filesystem
        Ok(bool_to_jboolean(true))
    })
}

/// Get number of books in library
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeGetBookCount(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    jni_safe!(env, -1, {
        LIBRARY_HANDLES.get(handle)?;

        // Placeholder: Would return actual count
        Ok(0)
    })
}

/// Get library root path
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeGetRootPath(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;

        // Placeholder: Would return actual path
        string_to_jstring(&mut env, "/placeholder/path")
    })
}

/// Get library size in bytes
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeGetLibrarySize(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jlong {
    jni_safe!(env, 0, {
        LIBRARY_HANDLES.get(handle)?;

        // Placeholder: Would calculate actual size
        Ok(0)
    })
}

/// Add audiobook from file path
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeAddBook(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    file_path: jstring,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let path = jstring_raw_to_string(&mut env, file_path)?;

        crate::ffi::log_info("StoryStream", &format!("Adding book from: {}", path));

        // Placeholder: Would parse metadata and add to library
        string_to_jstring(&mut env, "placeholder-book-id")
    })
}

/// Get book title by ID
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeGetBookTitle(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: jstring,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_raw_to_string(&mut env, book_id)?;

        crate::ffi::log_info("StoryStream", &format!("Getting title for book: {}", id));

        // Placeholder: Would query database
        string_to_jstring(&mut env, "Unknown Title")
    })
}

/// Get book author by ID
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeGetBookAuthor(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: jstring,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_raw_to_string(&mut env, book_id)?;

        crate::ffi::log_info("StoryStream", &format!("Getting author for book: {}", id));

        // Placeholder: Would query database
        option_string_to_jstring(&mut env, Some("Unknown Author"))
    })
}

/// Get book narrator by ID
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeGetBookNarrator(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: jstring,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_raw_to_string(&mut env, book_id)?;

        crate::ffi::log_info("StoryStream", &format!("Getting narrator for book: {}", id));

        // Placeholder: Would query database
        Ok(std::ptr::null_mut()) // Return null for no narrator
    })
}

/// Get book file path by ID
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeGetBookPath(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: jstring,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_raw_to_string(&mut env, book_id)?;

        crate::ffi::log_info("StoryStream", &format!("Getting path for book: {}", id));

        // Placeholder: Would query database
        string_to_jstring(&mut env, "/path/to/audiobook.mp3")
    })
}

/// Get book cover art path by ID
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeGetBookCoverArt(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: jstring,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_raw_to_string(&mut env, book_id)?;

        crate::ffi::log_info("StoryStream", &format!("Getting cover art for book: {}", id));

        // Placeholder: Would query database
        Ok(std::ptr::null_mut()) // Return null for no cover art
    })
}

/// Search library with query string
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeSearchBooks(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    query: jstring,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;
        let query_str = jstring_raw_to_string(&mut env, query)?;

        crate::ffi::log_info("StoryStream", &format!("Searching library: {}", query_str));

        // Placeholder: Would search database and return JSON array
        string_to_jstring(&mut env, "[]")
    })
}

/// Remove book from library
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeRemoveBook(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: jstring,
) -> jboolean {
    jni_safe!(env, bool_to_jboolean(false), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_raw_to_string(&mut env, book_id)?;

        crate::ffi::log_info("StoryStream", &format!("Removing book: {}", id));

        // Placeholder: Would remove from database
        Ok(bool_to_jboolean(true))
    })
}

/// Get all book IDs in library (JSON array)
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeGetAllBookIds(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;

        // Placeholder: Would query database
        string_to_jstring(&mut env, "[]")
    })
}

/// Get recently added books (JSON array)
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeGetRecentBooks(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    limit: jint,
) -> jstring {
    jni_safe!(env, std::ptr::null_mut(), {
        LIBRARY_HANDLES.get(handle)?;

        if limit < 0 {
            return Err(FfiError::General("Limit must be non-negative".to_string()));
        }

        crate::ffi::log_info("StoryStream", &format!("Getting {} recent books", limit));

        // Placeholder: Would query database
        string_to_jstring(&mut env, "[]")
    })
}

/// Update book metadata
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamLibrary_nativeUpdateBookMetadata(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    book_id: jstring,
) -> jboolean {
    jni_safe!(env, bool_to_jboolean(false), {
        LIBRARY_HANDLES.get(handle)?;
        let id = jstring_raw_to_string(&mut env, book_id)?;

        crate::ffi::log_info("StoryStream", &format!("Updating metadata for book: {}", id));

        // Placeholder: Would update database
        Ok(bool_to_jboolean(true))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_context_creation() {
        let ctx = LibraryContext::new("/test/path".to_string());
        assert_eq!(ctx.root_path, "/test/path");
        assert!(ctx.initialized);
    }

    #[test]
    fn test_handle_lifecycle() {
        let ctx = LibraryContext::new("/test".to_string());
        let handle = LIBRARY_HANDLES.insert(ctx.clone());
        assert!(handle > 0);

        let retrieved = LIBRARY_HANDLES.get(handle);
        assert!(retrieved.is_ok());

        let removed = LIBRARY_HANDLES.remove(handle);
        assert!(removed.is_ok());

        let not_found = LIBRARY_HANDLES.get(handle);
        assert!(not_found.is_err());
    }
}