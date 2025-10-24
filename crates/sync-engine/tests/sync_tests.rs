// crates/sync-engine/tests/sync_tests.rs
//! Integration tests for sync engine

use storystream_sync_engine::{
    ChangeType, ConflictResolution, DeviceId, EntityType, SyncConfig, SyncEngine, SyncResponse,
};

#[test]
fn test_basic_sync_workflow() {
    let config = SyncConfig::default();
    let engine = SyncEngine::new(config);

    // Record some changes
    engine
        .record_change(
            ChangeType::Update,
            EntityType::Position,
            "book-1".to_string(),
            serde_json::json!({"position": 1000}),
        )
        .unwrap();

    engine
        .record_change(
            ChangeType::Create,
            EntityType::Bookmark,
            "bookmark-1".to_string(),
            serde_json::json!({"title": "Important", "position": 1000}),
        )
        .unwrap();

    // Check state
    let state = engine.state().unwrap();
    assert_eq!(state.pending_changes, 2);

    // Simulate sync
    let remote_changes = vec![];
    let result = engine.sync(remote_changes).unwrap();
    assert_eq!(result.len(), 2);

    // State should be updated
    let state = engine.state().unwrap();
    assert_eq!(state.pending_changes, 0);
}

#[test]
fn test_conflict_resolution_newest_wins() {
    let config = SyncConfig {
        conflict_resolution: ConflictResolution::UseNewest,
        ..Default::default()
    };
    let engine = SyncEngine::new(config);

    // Record older local change
    engine
        .record_change(
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        )
        .unwrap();

    // Wait a bit
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Create newer remote change
    let remote_device = DeviceId::new();
    let remote_change = storystream_sync_engine::Change::new(
        remote_device,
        ChangeType::Update,
        EntityType::Position,
        "book-123".to_string(),
        serde_json::json!({"position": 2000}),
    );

    let result = engine.sync(vec![remote_change.clone()]).unwrap();

    // Should have only one change (the newer one)
    assert_eq!(result.len(), 1);
    // The position should be 2000 (from remote, which is newer)
    assert_eq!(result[0].data["position"], 2000);
}

#[test]
fn test_multiple_devices_sync() {
    // Device 1
    let config1 = SyncConfig::default();
    let engine1 = SyncEngine::new(config1);

    // Device 2
    let config2 = SyncConfig::default();
    let engine2 = SyncEngine::new(config2);

    // Device 1 records changes
    engine1
        .record_change(
            ChangeType::Update,
            EntityType::Position,
            "book-1".to_string(),
            serde_json::json!({"position": 1000}),
        )
        .unwrap();

    // Device 2 records changes
    engine2
        .record_change(
            ChangeType::Update,
            EntityType::Position,
            "book-2".to_string(),
            serde_json::json!({"position": 2000}),
        )
        .unwrap();

    // Create sync requests
    let request1 = engine1.create_sync_request().unwrap();
    let request2 = engine2.create_sync_request().unwrap();

    // Simulate server merging and responding
    let mut all_changes = request1.changes.clone();
    all_changes.extend(request2.changes.clone());

    // Both devices process responses
    let response1 = SyncResponse::success(request2.changes);
    let response2 = SyncResponse::success(request1.changes);

    let result1 = engine1.process_sync_response(response1).unwrap();
    let result2 = engine2.process_sync_response(response2).unwrap();

    // Both should have changes from the other device
    assert!(!result1.is_empty());
    assert!(!result2.is_empty());
}

#[test]
fn test_sync_request_creation() {
    let config = SyncConfig::default();
    let engine = SyncEngine::new(config);

    engine
        .record_change(
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        )
        .unwrap();

    let request = engine.create_sync_request().unwrap();

    assert_eq!(request.device_id, engine.device_id().to_string());
    assert_eq!(request.changes.len(), 1);
    assert!(request.since.is_some());
}

#[test]
fn test_sync_error_handling() {
    let config = SyncConfig::default();
    let engine = SyncEngine::new(config);

    let error_response = SyncResponse::error("Network timeout".to_string());
    let result = engine.process_sync_response(error_response);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Network timeout"));
}

#[test]
fn test_change_type_serialization() {
    let change_type = ChangeType::Update;
    let json = serde_json::to_string(&change_type).unwrap();
    let deserialized: ChangeType = serde_json::from_str(&json).unwrap();
    assert_eq!(change_type, deserialized);
}

#[test]
fn test_entity_type_serialization() {
    let entity_type = EntityType::Position;
    let json = serde_json::to_string(&entity_type).unwrap();
    let deserialized: EntityType = serde_json::from_str(&json).unwrap();
    assert_eq!(entity_type, deserialized);
}
