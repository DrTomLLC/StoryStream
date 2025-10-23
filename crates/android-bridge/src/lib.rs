// crates/android-bridge/src/lib.rs
//! Android Bridge for StoryStream
//!
//! This crate provides JNI (Java Native Interface) bindings to expose StoryStream's
//! Rust functionality to Android applications. It acts as the bridge between the
//! Android Java/Kotlin code and the core Rust audiobook player engine.
//!
//! # Architecture
//!
//! The bridge is organized into several modules:
//!
//! - `ffi`: Core FFI utilities, error handling, and handle management
//! - `player_bridge`: Audio playback controls (play, pause, seek, chapters, bookmarks)
//! - `library_bridge`: Library management (scanning, importing, metadata)
//!
//! # Safety
//!
//! All JNI functions follow safety-critical patterns:
//! - Comprehensive error handling with graceful degradation
//! - Panic catching to prevent crashes
//! - Handle-based resource management to prevent memory leaks
//! - Input validation on all boundaries
//!
//! # Usage from Android
//!
//! ## Player Example (Java)
//!
//! ```java
//! public class Player {
//!     private long nativeHandle;
//!     
//!     public Player(String audioFilePath) {
//!         nativeHandle = nativeCreate(audioFilePath);
//!     }
//!     
//!     public void play() {
//!         nativePlay(nativeHandle);
//!     }
//!     
//!     public void pause() {
//!         nativePause(nativeHandle);
//!     }
//!     
//!     public void destroy() {
//!         nativeDestroy(nativeHandle);
//!         nativeHandle = 0;
//!     }
//!     
//!     private native long nativeCreate(String audioFilePath);
//!     private native void nativeDestroy(long handle);
//!     private native boolean nativePlay(long handle);
//!     private native boolean nativePause(long handle);
//! }
//! ```
//!
//! ## Library Example (Java)
//!
//! ```java
//! public class Library {
//!     private long nativeHandle;
//!     
//!     public Library(String libraryPath) {
//!         nativeHandle = nativeCreate(libraryPath);
//!     }
//!     
//!     public void startScan() {
//!         nativeStartScan(nativeHandle);
//!     }
//!     
//!     public int getBookCount() {
//!         return nativeGetBookCount(nativeHandle);
//!     }
//!     
//!     public void destroy() {
//!         nativeDestroy(nativeHandle);
//!         nativeHandle = 0;
//!     }
//!     
//!     private native long nativeCreate(String libraryPath);
//!     private native void nativeDestroy(long handle);
//!     private native boolean nativeStartScan(long handle);
//!     private native int nativeGetBookCount(long handle);
//! }
//! ```
//!
//! # Thread Safety
//!
//! All exported functions are thread-safe. However, Android's JNI calls typically
//! occur on the UI thread, so long-running operations should be wrapped in async
//! tasks on the Java/Kotlin side.
//!
//! # Memory Management
//!
//! Rust objects are managed via opaque handles (i64 integers). The Android code
//! is responsible for:
//! 1. Storing the handle returned from `nativeCreate` functions
//! 2. Passing this handle to all subsequent operations
//! 3. Calling `nativeDestroy` when done to free resources
//!
//! Failure to call destroy functions will result in memory leaks.
//!
//! # Error Handling
//!
//! Errors are propagated to Java as RuntimeExceptions. The Android code should
//! catch these exceptions and handle them appropriately.
//!
//! # Testing
//!
//! This module includes unit tests for the handle management system. Integration
//! tests should be performed from the Android side using Android instrumentation
//! tests or similar frameworks.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::cargo)]

pub mod ffi;
pub mod library_bridge;
pub mod player_bridge;

// Re-export key types for convenience
pub use ffi::{FfiError, FfiResult};

/// Initialize the Android logger
///
/// This should be called once at application startup to configure
/// logging to Android's logcat system.
///
/// # Safety
///
/// This function is safe to call multiple times; subsequent calls are no-ops.
pub fn init_logger() {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_min_level(log::Level::Debug)
                .with_tag("StoryStream"),
        );
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }
}

/// Gets the version of the StoryStream library
///
/// # Safety
///
/// This function is safe to call from JNI.
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStream_nativeGetVersion(
    mut env: jni::JNIEnv,
    _class: jni::objects::JClass,
) -> jni::sys::jstring {
    crate::jni_safe!(env, std::ptr::null_mut(), {
        let version = env!("CARGO_PKG_VERSION");
        ffi::string_to_jstring(&mut env, version)
    })
}

/// Initializes the StoryStream native library
///
/// This should be called once when the Android application starts.
///
/// # Safety
///
/// This function is safe to call from JNI. It is safe to call multiple times.
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStream_nativeInit(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
) {
    init_logger();
    ffi::log_info("StoryStream", "Native library initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_init() {
        // Should not panic
        init_logger();
        init_logger(); // Multiple calls should be safe
    }

    #[test]
    fn test_version() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
    }
}