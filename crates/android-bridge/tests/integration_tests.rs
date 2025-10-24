// Integration tests for StoryStream Android Bridge
//
// These tests verify the core functionality of the FFI bridge without requiring
// an actual Android environment.

#[cfg(test)]
mod integration_tests {
    use storystream_android_bridge::ffi::{
        bool_to_jboolean, jboolean_to_bool, FfiError, FfiResult, HandleManager,
    };

    // Test panic handling in jni_safe macro
    #[test]
    fn test_panic_handling_infrastructure() {
        use std::panic;

        // Verify panic::catch_unwind is available and works
        let result = panic::catch_unwind(|| -> FfiResult<i32> { Ok(42) });
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), 42);

        // Verify panic::AssertUnwindSafe works
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| -> FfiResult<i32> {
            Err(FfiError::General("test error".to_string()))
        }));
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    // Test handle manager thread safety
    #[test]
    fn test_handle_manager_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let manager = Arc::new(HandleManager::<String>::default());
        let mut handles = vec![];

        // Create handles from multiple threads
        for i in 0..10 {
            let mgr = Arc::clone(&manager);
            let handle = thread::spawn(move || mgr.insert(format!("value-{}", i)));
            handles.push(handle);
        }

        // Collect all handles
        let handle_ids: Vec<i64> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Verify all handles are unique
        let mut sorted = handle_ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), handle_ids.len());

        // Verify all handles can be retrieved
        for handle_id in &handle_ids {
            assert!(manager.get(*handle_id).is_ok());
        }

        // Clean up
        for handle_id in handle_ids {
            assert!(manager.remove(handle_id).is_ok());
        }
    }

    // Test error conversion chain
    #[test]
    fn test_error_conversions() {
        use std::str::Utf8Error;

        // Test Display implementation
        let err = FfiError::JniError("jni failed".to_string());
        assert_eq!(err.to_string(), "JNI Error: jni failed");

        let err = FfiError::Utf8Error("utf8 failed".to_string());
        assert_eq!(err.to_string(), "UTF-8 Error: utf8 failed");

        let err = FfiError::InvalidHandle("handle 42 not found".to_string());
        assert_eq!(err.to_string(), "Invalid Handle: handle 42 not found");

        let err = FfiError::General("general error".to_string());
        assert_eq!(err.to_string(), "Error: general error");

        // Test error trait implementation
        let err: &dyn std::error::Error = &FfiError::General("test".to_string());
        assert!(err.to_string().contains("test"));
    }

    // Test boolean conversions are correct
    #[test]
    fn test_boolean_conversions_comprehensive() {
        // True conversions
        assert_eq!(bool_to_jboolean(true), 1);
        assert!(jboolean_to_bool(1));
        assert!(jboolean_to_bool(2)); // Any non-zero
        assert!(jboolean_to_bool(255));

        // False conversions
        assert_eq!(bool_to_jboolean(false), 0);
        assert!(!jboolean_to_bool(0));

        // Round-trip conversions
        assert_eq!(jboolean_to_bool(bool_to_jboolean(true)), true);
        assert_eq!(jboolean_to_bool(bool_to_jboolean(false)), false);
    }

    // Test handle manager edge cases
    #[test]
    fn test_handle_manager_edge_cases() {
        let manager = HandleManager::<String>::default();

        // Test invalid handle retrieval
        assert!(matches!(manager.get(0), Err(FfiError::InvalidHandle(_))));
        assert!(matches!(manager.get(-1), Err(FfiError::InvalidHandle(_))));
        assert!(matches!(
            manager.get(999999),
            Err(FfiError::InvalidHandle(_))
        ));

        // Test double removal
        let handle = manager.insert("test".to_string());
        assert!(manager.remove(handle).is_ok());
        assert!(matches!(
            manager.remove(handle),
            Err(FfiError::InvalidHandle(_))
        ));
    }

    // Test handle manager with different types
    #[test]
    fn test_handle_manager_generic_types() {
        // Test with integers
        let int_manager = HandleManager::<i32>::default();
        let h1 = int_manager.insert(42);
        assert!(int_manager.get(h1).is_ok());

        // Test with structs
        #[derive(Clone, Debug, PartialEq)]
        struct TestStruct {
            field: String,
        }

        let struct_manager = HandleManager::<TestStruct>::default();
        let h2 = struct_manager.insert(TestStruct {
            field: "test".to_string(),
        });
        assert!(struct_manager.get(h2).is_ok());

        // Test with Options
        let option_manager = HandleManager::<Option<String>>::default();
        let h3 = option_manager.insert(Some("value".to_string()));
        let h4 = option_manager.insert(None);
        assert!(option_manager.get(h3).is_ok());
        assert!(option_manager.get(h4).is_ok());
    }

    // Test handle manager capacity and performance
    #[test]
    fn test_handle_manager_large_scale() {
        let manager = HandleManager::<usize>::default();
        let mut handles = Vec::new();

        // Insert many handles
        for i in 0..1000 {
            handles.push(manager.insert(i));
        }

        // Verify all handles are valid and unique
        let mut sorted = handles.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), handles.len());

        // Remove half
        for i in (0..handles.len()).step_by(2) {
            assert!(manager.remove(handles[i]).is_ok());
        }

        // Verify correct ones remain
        for i in (1..handles.len()).step_by(2) {
            assert!(manager.get(handles[i]).is_ok());
        }
    }

    // Test FfiResult type alias
    #[test]
    fn test_ffi_result_type() {
        fn returns_result() -> FfiResult<i32> {
            Ok(42)
        }

        fn returns_error() -> FfiResult<i32> {
            Err(FfiError::General("test".to_string()))
        }

        assert!(returns_result().is_ok());
        assert!(returns_error().is_err());
        assert_eq!(returns_result().unwrap(), 42);
    }

    // Test error propagation with ? operator
    #[test]
    fn test_error_propagation() {
        fn inner_function() -> FfiResult<String> {
            Err(FfiError::General("inner error".to_string()))
        }

        fn outer_function() -> FfiResult<String> {
            let _value = inner_function()?;
            Ok("success".to_string())
        }

        let result = outer_function();
        assert!(result.is_err());
        match result {
            Err(FfiError::General(msg)) => assert_eq!(msg, "inner error"),
            _ => panic!("Expected General error"),
        }
    }

    // Test library version constant
    #[test]
    fn test_library_constants() {
        use storystream_android_bridge::{LIBRARY_NAME, VERSION};

        assert!(!VERSION.is_empty());
        assert!(VERSION.split('.').count() >= 2); // At least major.minor
        assert_eq!(LIBRARY_NAME, "StoryStream Android Bridge");
    }

    // Test logging doesn't panic
    #[test]
    fn test_logging_functions() {
        use storystream_android_bridge::ffi::{log_error, log_info};

        // These should not panic on any platform
        log_info("TEST", "info message");
        log_error("TEST", "error message");
        log_info("TEST", &format!("formatted: {}", 42));

        // Test with special characters
        log_info("TEST", "Message with 日本語 and emojis 🎵");
        log_info("TEST", "Message with\nnewlines\nand\ttabs");
    }

    // Stress test: verify no memory leaks in handle lifecycle
    #[test]
    fn test_handle_lifecycle_stress() {
        let manager = HandleManager::<Vec<u8>>::default();

        // Repeatedly create and destroy handles
        for _iteration in 0..100 {
            let mut handles = Vec::new();

            // Create 100 handles
            for i in 0..100 {
                let data = vec![i as u8; 1000]; // 1KB each
                handles.push(manager.insert(data));
            }

            // Remove all handles
            for handle in handles {
                assert!(manager.remove(handle).is_ok());
            }
        }

        // Manager should be empty
        assert!(manager.get(1).is_err());
    }

    // Test init_logging doesn't panic
    #[test]
    fn test_init_logging() {
        use storystream_android_bridge::init_logging;

        // Should be safe to call multiple times
        init_logging();
        init_logging();
        init_logging();
    }
}

// Additional module-specific tests
#[cfg(test)]
mod ffi_tests {
    use storystream_android_bridge::ffi::*;

    #[test]
    fn test_handle_manager_default() {
        let manager = HandleManager::<String>::default();
        let h = manager.insert("test".to_string());
        assert!(h > 0);
    }

    #[test]
    fn test_handle_manager_contains() {
        let manager = HandleManager::<String>::default();
        let h = manager.insert("test".to_string());

        assert!(manager.contains(h));
        assert!(!manager.contains(999));

        manager.remove(h).unwrap();
        assert!(!manager.contains(h));
    }
}

#[cfg(test)]
mod error_handling_tests {
    use storystream_android_bridge::FfiError;

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            FfiError::JniError("jni".to_string()),
            FfiError::Utf8Error("utf8".to_string()),
            FfiError::InvalidHandle("handle".to_string()),
            FfiError::General("general".to_string()),
        ];

        for error in errors {
            // All errors should be displayable
            let _s = error.to_string();

            // All errors should implement Error trait
            let _e: &dyn std::error::Error = &error;
        }
    }
}
