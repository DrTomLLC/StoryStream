//! Fuzzing harness for config parsing
//!
//! This fuzzer tests that arbitrary input never causes panics.
//! Run with: cargo fuzz run config_parser

#![no_main]
use libfuzzer_sys::fuzz_target;
use storystream_config::Config;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as TOML config
    // This should never panic, only return errors
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = toml::from_str::<Config>(s);
    }
});