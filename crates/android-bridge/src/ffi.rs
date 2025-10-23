// crates/android-bridge/src/ffi.rs
//! Core FFI infrastructure for JNI bindings
//!
//! This module provides the foundation for exposing Rust functionality to Android.
//! All JNI functions follow safety-critical patterns with comprehensive error handling.

use jni::objects::JString;
use jni::sys::{jboolean, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use std::ptr;

/// Result type for FFI operations
pub type FfiResult<T> = Result<T, FfiError>;

/// Errors that can occur in FFI operations
#[derive(Debug, thiserror::Error)]
pub enum FfiError {
    #[error("JNI error: {0}")]
    Jni(#[from] jni::errors::Error),

    #[error("Null pointer encountered")]
    NullPointer,

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("String conversion error")]
    StringConversion,

    #[error("Invalid handle: {0}")]
    InvalidHandle(i64),

    #[error("Panic caught: {0}")]
    Panic(String),

    #[error("Core error: {0}")]
    Core(String),
}

/// Converts a Java string to a Rust String
pub fn jstring_to_string(env: &mut JNIEnv, jstr: JString) -> FfiResult<String> {
    if jstr.is_null() {
        return Err(FfiError::NullPointer);
    }

    let java_str = env.get_string(&jstr)?;
    Ok(java_str.to_str()?.to_string())
}

/// Converts a Rust string to a Java string
pub fn string_to_jstring(env: &mut JNIEnv, s: &str) -> FfiResult<jstring> {
    let jstr = env.new_string(s)?;
    Ok(jstr.into_raw())
}

/// Converts a Rust Option<String> to a nullable Java string
pub fn option_string_to_jstring(env: &mut JNIEnv, opt: Option<&str>) -> FfiResult<jstring> {
    match opt {
        Some(s) => string_to_jstring(env, s),
        None => Ok(ptr::null_mut()),
    }
}

/// Converts a Rust bool to a Java boolean
pub fn bool_to_jboolean(b: bool) -> jboolean {
    if b {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// Converts a Java boolean to a Rust bool
pub fn jboolean_to_bool(jb: jboolean) -> bool {
    jb == JNI_TRUE
}

/// Macro to wrap JNI functions with panic handling
///
/// This ensures that any Rust panics are caught and converted to Java exceptions
/// rather than crashing the entire Android app.
#[macro_export]
macro_rules! jni_safe {
    ($env:expr, $default:expr, $body:block) => {{
        match panic::catch_unwind(panic::AssertUnwindSafe(|| -> FfiResult<_> { $body })) {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                let _ = $env.throw_new("java/lang/RuntimeException", format!("{}", e));
                $default
            }
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    format!("Rust panic: {}", s)
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    format!("Rust panic: {}", s)
                } else {
                    "Rust panic: unknown error".to_string()
                };
                let _ = $env.throw_new("java/lang/RuntimeException", msg);
                $default
            }
        }
    }};
}

/// Logs an error message to Android logcat
pub fn log_error(tag: &str, message: &str) {
    #[cfg(target_os = "android")]
    {
        use android_logger::log;
        log::error!(target: tag, "{}", message);
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[{}] ERROR: {}", tag, message);
    }
}

/// Logs an info message to Android logcat
pub fn log_info(tag: &str, message: &str) {
    #[cfg(target_os = "android")]
    {
        use android_logger::log;
        log::info!(target: tag, "{}", message);
    }
    #[cfg(not(target_os = "android"))]
    {
        println!("[{}] INFO: {}", tag, message);
    }
}

/// Logs a debug message to Android logcat
pub fn log_debug(tag: &str, message: &str) {
    #[cfg(target_os = "android")]
    {
        use android_logger::log;
        log::debug!(target: tag, "{}", message);
    }
    #[cfg(not(target_os = "android"))]
    {
        println!("[{}] DEBUG: {}", tag, message);
    }
}

/// Handle manager for tracking native objects
///
/// This provides a safe way to pass Rust objects across the FFI boundary
/// by converting them to opaque integer handles that Java can store.
pub struct HandleManager<T> {
    next_handle: std::sync::atomic::AtomicI64,
    objects: std::sync::Mutex<std::collections::HashMap<i64, Box<T>>>,
}

impl<T> HandleManager<T> {
    /// Creates a new handle manager
    pub fn new() -> Self {
        Self {
            next_handle: std::sync::atomic::AtomicI64::new(1),
            objects: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Stores an object and returns a handle to it
    pub fn insert(&self, obj: T) -> i64 {
        let handle = self
            .next_handle
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut objects = self.objects.lock().unwrap();
        objects.insert(handle, Box::new(obj));
        handle
    }

    /// Retrieves a reference to an object by its handle
    ///
    /// Returns true if the handle exists, false otherwise.
    /// Use this for validation before operations.
    pub fn get(&self, handle: i64) -> FfiResult<()> {
        let objects = self.objects.lock().unwrap();
        if objects.contains_key(&handle) {
            Ok(())
        } else {
            Err(FfiError::InvalidHandle(handle))
        }
    }

    /// Executes a function with access to the object at the given handle
    pub fn with<F, R>(&self, handle: i64, f: F) -> FfiResult<R>
    where
        F: FnOnce(&T) -> R,
    {
        let objects = self.objects.lock().unwrap();
        objects
            .get(&handle)
            .map(|obj| f(obj.as_ref()))
            .ok_or(FfiError::InvalidHandle(handle))
    }

    /// Retrieves and removes an object by its handle
    pub fn remove(&self, handle: i64) -> FfiResult<Box<T>> {
        let mut objects = self.objects.lock().unwrap();
        objects
            .remove(&handle)
            .ok_or(FfiError::InvalidHandle(handle))
    }

    /// Checks if a handle exists
    pub fn contains(&self, handle: i64) -> bool {
        let objects = self.objects.lock().unwrap();
        objects.contains_key(&handle)
    }

    /// Returns the number of managed objects
    pub fn len(&self) -> usize {
        let objects = self.objects.lock().unwrap();
        objects.len()
    }

    /// Checks if there are no managed objects
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Default for HandleManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_conversion() {
        assert_eq!(bool_to_jboolean(true), JNI_TRUE);
        assert_eq!(bool_to_jboolean(false), JNI_FALSE);
        assert!(jboolean_to_bool(JNI_TRUE));
        assert!(!jboolean_to_bool(JNI_FALSE));
    }

    #[test]
    fn test_handle_manager() {
        let manager = HandleManager::<String>::new();

        // Insert some objects
        let handle1 = manager.insert("Hello".to_string());
        let handle2 = manager.insert("World".to_string());

        assert_ne!(handle1, handle2);
        assert_eq!(manager.len(), 2);
        assert!(manager.contains(handle1));
        assert!(manager.contains(handle2));

        // Remove an object
        let obj = manager.remove(handle1).unwrap();
        assert_eq!(*obj, "Hello");
        assert_eq!(manager.len(), 1);
        assert!(!manager.contains(handle1));

        // Try to remove non-existent handle
        assert!(manager.remove(handle1).is_err());
    }

    #[test]
    fn test_handle_manager_empty() {
        let manager = HandleManager::<i32>::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);

        let handle = manager.insert(42);
        assert!(!manager.is_empty());
        assert_eq!(manager.len(), 1);

        manager.remove(handle).unwrap();
        assert!(manager.is_empty());
    }
}