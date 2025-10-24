// StoryStream Android Bridge
//
// JNI bridge for audiobook player and library management on Android.
// This library provides FFI-safe bindings between Rust core logic and Java/Kotlin Android code.

#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(target_os = "android", allow(dead_code))]

// Module declarations
pub mod ffi;
pub mod library_bridge;
pub mod player_bridge;

// Re-export key types for convenience
pub use ffi::{FfiError, FfiResult, HandleManager};

use jni::{objects::JClass, sys::jstring, JNIEnv};
use std::panic; // Required for jni_safe! macro

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const LIBRARY_NAME: &str = "StoryStream Android Bridge";

/// Initialize logging for the library
///
/// This should be called once when the library is loaded.
/// On Android, logs will go to logcat. On other platforms, logs go to stderr.
pub fn init_logging() {
    #[cfg(target_os = "android")]
    {
        android_log::init("StoryStream").unwrap();
        crate::ffi::log_info("StoryStream", "Logging initialized for Android");
    }

    #[cfg(not(target_os = "android"))]
    {
        // Simple stderr logging for non-Android platforms (testing/development)
        eprintln!("[StoryStream] Logging initialized (stderr)");
    }
}

/// JNI_OnLoad - called when library is loaded
///
/// # Safety
/// This is called by the JVM and must follow JNI conventions
#[cfg(target_os = "android")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn JNI_OnLoad(
    _vm: jni::JavaVM,
    _reserved: *mut std::ffi::c_void,
) -> jni::sys::jint {
    init_logging();
    crate::ffi::log_info("StoryStream", &format!("Library loaded - version {}", VERSION));
    jni::JNIVersion::V6.into()
}

/// Get library version
///
/// # Safety
/// Must be called from Java with valid JNI environment
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStream_nativeGetVersion(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    crate::jni_safe!(env, std::ptr::null_mut(), {
        let version = env!("CARGO_PKG_VERSION");
        ffi::string_to_jstring(&mut env, version)
    })
}

/// Platform detection for build warnings
#[cfg(not(target_os = "android"))]
fn emit_warning() {
    println!("cargo:warning=Building for non-Android target: {} ({})",
             std::env::consts::OS,
             std::env::consts::ARCH);
    println!("cargo:warning=JNI functions will be available but may not work correctly outside Android");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.'));
    }

    #[test]
    fn test_library_name() {
        assert_eq!(LIBRARY_NAME, "StoryStream Android Bridge");
    }

    #[test]
    fn test_init_logging() {
        // Should not panic
        init_logging();
    }
}