// crates/network/tests/network_tests.rs
//! Integration tests for network module

use std::time::Duration;
use storystream_network::{
    Client, ClientConfig, ConnectivityChecker, DownloadManager, ProgressTracker,
};

#[tokio::test]
async fn test_client_basic_operations() {
    let client = Client::new().expect("Failed to create client");

    // Test that client can be created and cloned
    let _cloned = client.clone();
}

#[tokio::test]
async fn test_client_with_custom_config() {
    let config = ClientConfig {
        timeout: Duration::from_secs(10),
        user_agent: "TestClient/1.0".to_string(),
        max_redirects: 5,
        retry_policy: None,
        circuit_breaker_config: None,
    };

    let client = Client::with_config(config);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_download_manager_creation() {
    let client = Client::new().expect("Failed to create client");
    let _manager = DownloadManager::new(client);
}

#[tokio::test]
async fn test_progress_tracker() {
    let tracker = ProgressTracker::new(Some(10000));

    tracker.update(2500);
    assert_eq!(tracker.percentage(), Some(25.0));

    tracker.update(2500);
    assert_eq!(tracker.percentage(), Some(50.0));

    tracker.update(5000);
    assert!(tracker.is_complete());
}

#[tokio::test]
async fn test_connectivity_checker_creation() {
    let client = Client::new().expect("Failed to create client");
    let _checker = ConnectivityChecker::new(client);
}

#[tokio::test]
async fn test_connectivity_with_custom_urls() {
    let client = Client::new().expect("Failed to create client");
    let urls = vec!["https://www.rust-lang.org".to_string()];
    let _checker = ConnectivityChecker::with_urls(client, urls);
}

#[tokio::test]
async fn test_progress_unknown_size() {
    let tracker = ProgressTracker::new(None);

    tracker.update(1000);
    assert_eq!(tracker.percentage(), None);
    assert!(!tracker.is_complete());
}

#[tokio::test]
async fn test_multiple_progress_updates() {
    let tracker = ProgressTracker::new(Some(1000));

    for _ in 0..10 {
        tracker.update(100);
    }

    assert!(tracker.is_complete());
    assert_eq!(tracker.percentage(), Some(100.0));
}

#[test]
fn test_client_clone() {
    let client = Client::new().expect("Failed to create client");
    let cloned = client.clone();

    // Both should work independently
    drop(client);
    drop(cloned);
}

#[test]
fn test_progress_tracker_thread_safety() {
    let tracker = ProgressTracker::new(Some(10000));
    let tracker_clone = tracker.clone();

    std::thread::spawn(move || {
        tracker_clone.update(5000);
    })
    .join()
    .expect("Thread panicked");

    // Wait a bit for thread to complete
    std::thread::sleep(Duration::from_millis(10));

    // Check progress was updated
    let progress = tracker.get().expect("Failed to get progress");
    assert_eq!(progress.downloaded_bytes, 5000);
}
