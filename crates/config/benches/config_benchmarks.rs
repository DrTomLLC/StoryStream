//! Benchmarks for configuration system
//!
//! Run with: cargo bench --package storystream-config

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use storystream_config::{Config, ConfigManager};
use tempfile::TempDir;

fn setup_test_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().to_path_buf();
    (temp_dir, config_path)
}

fn bench_config_creation(c: &mut Criterion) {
    c.bench_function("config_default", |b| {
        b.iter(|| {
            let config = Config::default();
            black_box(config);
        });
    });
}

fn bench_config_validation(c: &mut Criterion) {
    let config = Config::default();

    c.bench_function("config_validate", |b| {
        b.iter(|| {
            let result = config.validate();
            black_box(result);
        });
    });
}

fn bench_config_serialization(c: &mut Criterion) {
    let config = Config::default();

    c.bench_function("config_serialize_toml", |b| {
        b.iter(|| {
            let toml = toml::to_string(&config).expect("Failed to serialize");
            black_box(toml);
        });
    });
}

fn bench_config_deserialization(c: &mut Criterion) {
    let config = Config::default();
    let toml_string = toml::to_string(&config).expect("Failed to serialize");

    c.bench_function("config_deserialize_toml", |b| {
        b.iter(|| {
            let config: Config = toml::from_str(&toml_string).expect("Failed to deserialize");
            black_box(config);
        });
    });
}

fn bench_config_save(c: &mut Criterion) {
    let (_temp_dir, config_dir) = setup_test_dir();
    let manager = ConfigManager::with_directory(config_dir).expect("Failed to create manager");
    let config = Config::default();

    c.bench_function("config_save", |b| {
        b.iter(|| {
            manager.save(&config).expect("Failed to save");
        });
    });
}

fn bench_config_load(c: &mut Criterion) {
    let (_temp_dir, config_dir) = setup_test_dir();
    let manager = ConfigManager::with_directory(config_dir).expect("Failed to create manager");
    let config = Config::default();
    manager.save(&config).expect("Failed to save");

    c.bench_function("config_load", |b| {
        b.iter(|| {
            let loaded = manager.load().expect("Failed to load");
            black_box(loaded);
        });
    });
}

fn bench_config_merge(c: &mut Criterion) {
    let mut base = Config::default();
    let override_config = Config::default();

    c.bench_function("config_merge", |b| {
        b.iter(|| {
            base.merge(override_config.clone());
            black_box(&base);
        });
    });
}

fn bench_config_manager_initialization(c: &mut Criterion) {
    c.bench_function("config_manager_new", |b| {
        b.iter(|| {
            let manager = ConfigManager::new();
            black_box(manager);
        });
    });
}

fn bench_config_update(c: &mut Criterion) {
    let (_temp_dir, config_dir) = setup_test_dir();
    let manager = ConfigManager::with_directory(config_dir).expect("Failed to create manager");
    manager.initialize().expect("Failed to initialize");

    c.bench_function("config_update", |b| {
        b.iter(|| {
            manager
                .update(|config| {
                    config.player.default_volume = 80;
                })
                .expect("Failed to update");
        });
    });
}

criterion_group!(
    benches,
    bench_config_creation,
    bench_config_validation,
    bench_config_serialization,
    bench_config_deserialization,
    bench_config_save,
    bench_config_load,
    bench_config_merge,
    bench_config_manager_initialization,
    bench_config_update
);

criterion_main!(benches);
