// crates/sync-engine/examples/sync_demo.rs
//! Demonstration of sync engine capabilities

use storystream_sync_engine::{
    ChangeType, ConflictResolution, DeviceId, EntityType, SyncConfig, SyncEngine,
};

fn main() {
    println!("StoryStream Sync Engine Demo");
    println!("=============================\n");

    demo_basic_sync();
    println!();
    demo_conflict_resolution();
    println!();
    demo_multi_device();
}

fn demo_basic_sync() {
    println!("1. Basic Synchronization");
    println!("------------------------");

    let config = SyncConfig::default();
    let engine = SyncEngine::new(config);

    println!("Device ID: {}", engine.device_id());

    // Record some changes
    println!("\nRecording changes:");

    engine
        .record_change(
            ChangeType::Update,
            EntityType::Position,
            "book-moby-dick".to_string(),
            serde_json::json!({"position": 1500, "chapter": 5}),
        )
        .unwrap();
    println!("  ✓ Updated position for Moby Dick");

    engine
        .record_change(
            ChangeType::Create,
            EntityType::Bookmark,
            "bookmark-1".to_string(),
            serde_json::json!({"title": "Call me Ishmael", "position": 100}),
        )
        .unwrap();
    println!("  ✓ Created bookmark");

    let state = engine.state().unwrap();
    println!("\nPending changes: {}", state.pending_changes);

    // Simulate sync
    let result = engine.sync(vec![]).unwrap();
    println!("Synced {} changes", result.len());

    let state = engine.state().unwrap();
    println!("Pending changes after sync: {}", state.pending_changes);
}

fn demo_conflict_resolution() {
    println!("2. Conflict Resolution");
    println!("----------------------");

    let config = SyncConfig {
        device_id: DeviceId::from_string("laptop".to_string()),
        conflict_resolution: ConflictResolution::UseNewest,
        auto_sync: false,
    };
    let engine = SyncEngine::new(config);

    println!("Strategy: Use Newest");

    // Local change
    println!("\nLaptop: Setting position to 1000");
    engine
        .record_change(
            ChangeType::Update,
            EntityType::Position,
            "book-123".to_string(),
            serde_json::json!({"position": 1000}),
        )
        .unwrap();

    // Wait a bit
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Simulate remote change from phone
    println!("Phone: Setting position to 2000 (newer)");
    let phone_device = DeviceId::from_string("phone".to_string());
    let remote_change = storystream_sync_engine::Change::new(
        phone_device,
        ChangeType::Update,
        EntityType::Position,
        "book-123".to_string(),
        serde_json::json!({"position": 2000}),
    );

    let result = engine.sync(vec![remote_change]).unwrap();

    println!("\n✓ Conflict resolved");
    println!("Winner: Position = {}", result[0].data["position"]);
}

fn demo_multi_device() {
    println!("3. Multi-Device Sync");
    println!("--------------------");

    // Create two devices
    let laptop_config = SyncConfig {
        device_id: DeviceId::from_string("laptop".to_string()),
        ..Default::default()
    };
    let laptop = SyncEngine::new(laptop_config);

    let phone_config = SyncConfig {
        device_id: DeviceId::from_string("phone".to_string()),
        ..Default::default()
    };
    let phone = SyncEngine::new(phone_config);

    println!("Devices:");
    println!("  - Laptop: {}", laptop.device_id());
    println!("  - Phone: {}", phone.device_id());

    // Laptop makes changes
    println!("\nLaptop: Reading Moby Dick");
    laptop
        .record_change(
            ChangeType::Update,
            EntityType::Position,
            "moby-dick".to_string(),
            serde_json::json!({"position": 5000}),
        )
        .unwrap();

    // Phone makes changes
    println!("Phone: Reading Pride and Prejudice");
    phone
        .record_change(
            ChangeType::Update,
            EntityType::Position,
            "pride-prejudice".to_string(),
            serde_json::json!({"position": 3000}),
        )
        .unwrap();

    // Create sync requests
    let laptop_request = laptop.create_sync_request().unwrap();
    let phone_request = phone.create_sync_request().unwrap();

    println!("\nSync Summary:");
    println!(
        "  Laptop has {} pending changes",
        laptop_request.changes.len()
    );
    println!(
        "  Phone has {} pending changes",
        phone_request.changes.len()
    );

    // Simulate server sync (each device gets other's changes)
    let laptop_response = storystream_sync_engine::SyncResponse::success(phone_request.changes);
    let phone_response = storystream_sync_engine::SyncResponse::success(laptop_request.changes);

    laptop.process_sync_response(laptop_response).unwrap();
    phone.process_sync_response(phone_response).unwrap();

    println!("\n✓ Devices synced successfully");
    println!("Both devices now have all reading progress!");
}
