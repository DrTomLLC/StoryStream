// FFI utility functions and error handling for JNI bridge
//
// This module provides safe wrappers for JNI operations with panic handling
// and type conversions between Rust and Java types.

use jni::{
    objects::{JClass, JString},
    sys::jstring,
    JNIEnv,
};
use std::ffi::CString;
use std::panic; // Required for catch_unwind in jni_safe! macro

/// FFI-safe error type that can cross the Rust/Java boundary
#[derive(Debug)]
pub enum FfiError {
    /// JNI operation failed
    JniError(String),
    /// Invalid UTF-8 string conversion
    Utf8Error(String),
    /// Handle not found or invalid
    InvalidHandle(String),
    /// General error
    General(String),
}

impl std::fmt::Display for FfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FfiError::JniError(msg) => write!(f, "JNI Error: {}", msg),
            FfiError::Utf8Error(msg) => write!(f, "UTF-8 Error: {}", msg),
            FfiError::InvalidHandle(msg) => write!(f, "Invalid Handle: {}", msg),
            FfiError::General(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for FfiError {}

impl From<jni::errors::Error> for FfiError {
    fn from(err: jni::errors::Error) -> Self {
        FfiError::JniError(err.to_string())
    }
}

impl From<std::str::Utf8Error> for FfiError {
    fn from(err: std::str::Utf8Error) -> Self {
        FfiError::Utf8Error(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for FfiError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        FfiError::Utf8Error(err.to_string())
    }
}

pub type FfiResult<T> = Result<T, FfiError>;

/// Thread-safe handle manager for storing and retrieving typed values
pub struct HandleManager<T> {
    handles: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<i64, T>>>,
    next_handle: std::sync::Arc<std::sync::atomic::AtomicI64>,
}

impl<T> Default for HandleManager<T> {
    fn default() -> Self {
        Self {
            handles: std::sync::Arc::new(std::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            next_handle: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(1)),
        }
    }
}

impl<T> HandleManager<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, value: T) -> i64 {
        let handle = self
            .next_handle
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.handles.write().unwrap().insert(handle, value);
        handle
    }

    pub fn get(&self, handle: i64) -> FfiResult<std::sync::Arc<std::sync::RwLock<T>>>
    where
        T: Clone,
    {
        let handles = self.handles.read().unwrap();
        handles
            .get(&handle)
            .cloned()
            .map(|v| std::sync::Arc::new(std::sync::RwLock::new(v)))
            .ok_or_else(|| FfiError::InvalidHandle(format!("Handle {} not found", handle)))
    }

    pub fn remove(&self, handle: i64) -> FfiResult<T> {
        self.handles
            .write()
            .unwrap()
            .remove(&handle)
            .ok_or_else(|| FfiError::InvalidHandle(format!("Handle {} not found", handle)))
    }

    pub fn contains(&self, handle: i64) -> bool {
        self.handles.read().unwrap().contains_key(&handle)
    }
}

/// Macro for safe JNI function calls with panic handling
///
/// This macro wraps JNI operations in catch_unwind to prevent panics from
/// crossing the FFI boundary, which would cause undefined behavior.
#[macro_export]
macro_rules! jni_safe {
    ($env:expr, $default:expr, $body:block) => {
        match panic::catch_unwind(panic::AssertUnwindSafe(|| -> FfiResult<_> { $body })) {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                let _ = $env.throw_new("java/lang/RuntimeException", e.to_string());
                $default
            }
            Err(panic_err) => {
                let msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                    format!("Panic: {}", s)
                } else if let Some(s) = panic_err.downcast_ref::<String>() {
                    format!("Panic: {}", s)
                } else {
                    "Unknown panic occurred".to_string()
                };
                let _ = $env.throw_new("java/lang/RuntimeException", msg);
                $default
            }
        }
    };
}

/// Convert Java string to Rust String
pub fn jstring_to_string(env: &mut JNIEnv, jstr: JString) -> FfiResult<String> {
    if jstr.is_null() {
        return Err(FfiError::General("Null JString provided".to_string()));
    }
    let java_str = env.get_string(&jstr)?;
    Ok(java_str.to_str()?.to_string())
}

/// Convert raw jstring pointer to Rust String (for JNI callbacks)
pub fn jstring_raw_to_string(env: &mut JNIEnv, jstr: jstring) -> FfiResult<String> {
    if jstr.is_null() {
        return Err(FfiError::General("Null jstring provided".to_string()));
    }
    // SAFETY: We trust the JNI contract that jstr is a valid JString
    // We wrap it to prevent double-free since JNI owns the actual object
    let jstring_obj = std::mem::ManuallyDrop::new(unsafe { JString::from_raw(jstr) });
    let java_str = env.get_string(&*jstring_obj)?;
    Ok(java_str.to_str()?.to_string())
}

/// Convert Rust String to Java string
pub fn string_to_jstring(env: &mut JNIEnv, s: &str) -> FfiResult<jstring> {
    let jstr = env.new_string(s)?;
    Ok(jstr.into_raw())
}

/// Convert Option<String> to Java string (null for None)
pub fn option_string_to_jstring(env: &mut JNIEnv, opt: Option<&str>) -> FfiResult<jstring> {
    match opt {
        Some(s) => string_to_jstring(env, s),
        None => Ok(std::ptr::null_mut()),
    }
}

/// Convert Rust bool to jboolean
pub fn bool_to_jboolean(b: bool) -> u8 {
    if b {
        1
    } else {
        0
    }
}

/// Convert jboolean to Rust bool
pub fn jboolean_to_bool(jb: u8) -> bool {
    jb != 0
}

/// Log info message to Android logcat
#[cfg(target_os = "android")]
pub fn log_info(tag: &str, message: &str) {
    android_log::log(android_log::Priority::Info, tag, message);
}

/// Fallback logging for non-Android platforms
#[cfg(not(target_os = "android"))]
pub fn log_info(tag: &str, message: &str) {
    eprintln!("[INFO][{}] {}", tag, message);
}

/// Log error message to Android logcat
#[cfg(target_os = "android")]
pub fn log_error(tag: &str, message: &str) {
    android_log::log(android_log::Priority::Error, tag, message);
}

/// Fallback logging for non-Android platforms
#[cfg(not(target_os = "android"))]
pub fn log_error(tag: &str, message: &str) {
    eprintln!("[ERROR][{}] {}", tag, message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_conversions() {
        assert_eq!(bool_to_jboolean(true), 1);
        assert_eq!(bool_to_jboolean(false), 0);
        assert!(jboolean_to_bool(1));
        assert!(jboolean_to_bool(255));
        assert!(!jboolean_to_bool(0));
    }

    #[test]
    fn test_handle_manager() {
        let manager = HandleManager::<String>::default();
        let handle = manager.insert("test".to_string());
        assert!(handle > 0);
        assert!(manager.get(handle).is_ok());
        assert!(manager.remove(handle).is_ok());
        assert!(manager.get(handle).is_err());
    }

    #[test]
    fn test_handle_manager_invalid() {
        let manager = HandleManager::<String>::default();
        assert!(manager.get(999).is_err());
        assert!(manager.remove(999).is_err());
    }
}