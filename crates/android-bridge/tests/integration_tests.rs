// crates/android-bridge/tests/integration_tests.rs
//! Integration tests for Android bridge
//!
//! These tests verify the FFI layer works correctly without requiring
//! an actual Android environment. They test the handle management,
//! error handling, and panic safety.

use storystream_android_bridge::{ffi::HandleManager, FfiError};
use std::sync::Arc;

#[test]
fn test_handle_manager_basic_operations() {
    let manager = HandleManager::<String>::new();

    // Test insertion
    let handle1 = manager.insert("First".to_string());
    let handle2 = manager.insert("Second".to_string());
    let handle3 = manager.insert("Third".to_string());

    // Handles should be unique
    assert_ne!(handle1, handle2);
    assert_ne!(handle2, handle3);
    assert_ne!(handle1, handle3);

    // All handles should exist
    assert!(manager.contains(handle1));
    assert!(manager.contains(handle2));
    assert!(manager.contains(handle3));
    assert_eq!(manager.len(), 3);
    assert!(!manager.is_empty());
}

#[test]
fn test_handle_manager_removal() {
    let manager = HandleManager::<i32>::new();

    let h1 = manager.insert(100);
    let h2 = manager.insert(200);
    let h3 = manager.insert(300);

    // Remove middle handle
    let value = manager.remove(h2).unwrap();
    assert_eq!(*value, 200);
    assert_eq!(manager.len(), 2);
    assert!(!manager.contains(h2));

    // Other handles should still be valid
    assert!(manager.contains(h1));
    assert!(manager.contains(h3));

    // Try to remove the same handle again - should fail
    let result = manager.remove(h2);
    assert!(result.is_err());

    // Remove remaining handles
    manager.remove(h1).unwrap();
    manager.remove(h3).unwrap();
    assert!(manager.is_empty());
    assert_eq!(manager.len(), 0);
}

#[test]
fn test_handle_manager_invalid_handle() {
    let manager = HandleManager::<String>::new();

    // Try to access non-existent handle
    let result = manager.get(99999);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FfiError::InvalidHandle(_)));

    // Try to remove non-existent handle
    let result = manager.remove(88888);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FfiError::InvalidHandle(_)));
}

#[test]
fn test_handle_manager_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(HandleManager::<i32>::new());
    let mut handles = Vec::new();

    // Spawn threads that insert values
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || manager_clone.insert(i * 10))
            .join()
            .unwrap();
        handles.push(handle);
    }

    // Verify all handles are valid
    assert_eq!(manager.len(), 10);
    for handle in &handles {
        assert!(manager.contains(*handle));
    }

    // Clean up
    for handle in handles {
        manager.remove(handle).unwrap();
    }
    assert!(manager.is_empty());
}

#[test]
fn test_handle_manager_with_complex_types() {
    struct ComplexType {
        id: String,
        data: Vec<u8>,
        flag: bool,
    }

    let manager = HandleManager::<ComplexType>::new();

    let obj = ComplexType {
        id: "test-123".to_string(),
        data: vec![1, 2, 3, 4, 5],
        flag: true,
    };

    let handle = manager.insert(obj);
    assert!(manager.contains(handle));

    let retrieved = manager.remove(handle).unwrap();
    assert_eq!(retrieved.id, "test-123");
    assert_eq!(retrieved.data, vec![1, 2, 3, 4, 5]);
    assert!(retrieved.flag);
}

#[test]
fn test_handle_manager_drop_behavior() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let drop_count = Arc::new(AtomicUsize::new(0));

    struct DropCounter {
        counter: Arc<AtomicUsize>,
    }

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    let manager = HandleManager::<DropCounter>::new();

    // Insert some objects
    let h1 = manager.insert(DropCounter {
        counter: Arc::clone(&drop_count),
    });
    let h2 = manager.insert(DropCounter {
        counter: Arc::clone(&drop_count),
    });
    let h3 = manager.insert(DropCounter {
        counter: Arc::clone(&drop_count),
    });

    assert_eq!(drop_count.load(Ordering::SeqCst), 0);

    // Remove one object
    manager.remove(h1).unwrap();
    assert_eq!(drop_count.load(Ordering::SeqCst), 1);

    // Remove remaining objects
    manager.remove(h2).unwrap();
    manager.remove(h3).unwrap();
    assert_eq!(drop_count.load(Ordering::SeqCst), 3);
}

#[test]
fn test_handle_manager_large_scale() {
    let manager = HandleManager::<usize>::new();
    let mut handles = Vec::new();

    // Insert 1000 items
    for i in 0..1000 {
        let handle = manager.insert(i);
        handles.push(handle);
    }

    assert_eq!(manager.len(), 1000);

    // Remove every other item
    for (idx, handle) in handles.iter().enumerate() {
        if idx % 2 == 0 {
            let value = manager.remove(*handle).unwrap();
            assert_eq!(*value, idx);
        }
    }

    assert_eq!(manager.len(), 500);

    // Remove remaining items
    for (idx, handle) in handles.iter().enumerate() {
        if idx % 2 != 0 {
            manager.remove(*handle).unwrap();
        }
    }

    assert!(manager.is_empty());
}

#[test]
fn test_ffi_error_types() {
    // Test that all error types can be created
    let _err1 = FfiError::NullPointer;
    let _err2 = FfiError::StringConversion;
    let _err3 = FfiError::InvalidHandle(123);
    let _err4 = FfiError::Panic("test panic".to_string());
    let _err5 = FfiError::Core("test error".to_string());

    // Test error display
    let err = FfiError::InvalidHandle(42);
    let msg = format!("{}", err);
    assert!(msg.contains("42"));
}

#[test]
fn test_logger_initialization() {
    // This should not panic
    storystream_android_bridge::init_logger();

    // Multiple calls should be safe
    storystream_android_bridge::init_logger();
    storystream_android_bridge::init_logger();
}

#[test]
fn test_module_compilation() {
    // This test just ensures all modules compile successfully
    // The actual JNI functions can't be tested without an Android environment

    // Verify that the library is accessible
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());
}

#[test]
fn test_thread_safety_with_arc() {
    use std::thread;

    let manager = Arc::new(HandleManager::<String>::new());
    let mut thread_handles = Vec::new();

    // Spawn 10 threads that each insert and remove items
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            let item_handle = manager_clone.insert(format!("Item {}", i));
            assert!(manager_clone.contains(item_handle));
            manager_clone.remove(item_handle).unwrap();
        });
        thread_handles.push(handle);
    }

    // Wait for all threads
    for handle in thread_handles {
        handle.join().unwrap();
    }

    // Manager should be empty
    assert!(manager.is_empty());
}

#[test]
fn test_handle_sequential_allocation() {
    let manager = HandleManager::<i32>::new();

    // Handles should be allocated sequentially
    let h1 = manager.insert(1);
    let h2 = manager.insert(2);
    let h3 = manager.insert(3);

    // They should increase
    assert!(h2 > h1);
    assert!(h3 > h2);

    // Even after removal, new handles should continue to increase
    manager.remove(h2).unwrap();
    let h4 = manager.insert(4);
    assert!(h4 > h3);
}

#[test]
fn test_zero_sized_types() {
    struct ZeroSized;

    let manager = HandleManager::<ZeroSized>::new();
    let h1 = manager.insert(ZeroSized);
    let h2 = manager.insert(ZeroSized);

    assert_ne!(h1, h2);
    assert_eq!(manager.len(), 2);

    manager.remove(h1).unwrap();
    manager.remove(h2).unwrap();
    assert!(manager.is_empty());
}

#[test]
fn test_error_propagation() {
    let manager = HandleManager::<String>::new();

    // Attempt operations on invalid handle
    let result = manager.get(12345);
    assert!(result.is_err());

    if let Err(e) = result {
        // Error should be InvalidHandle
        assert!(matches!(e, FfiError::InvalidHandle(12345)));

        // Error should be displayable
        let error_string = format!("{}", e);
        assert!(!error_string.is_empty());
    }
}