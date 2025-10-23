// crates/android-bridge/build.rs
//! Build script for android-bridge
//!
//! This script handles platform-specific configuration for building the
//! Android JNI bridge. It sets up appropriate compiler flags and links
//! against the necessary system libraries.

use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    println!("cargo:rerun-if-changed=build.rs");

    // Android-specific configuration
    if target_os == "android" {
        println!("cargo:rustc-link-lib=log");  // Android logging library
        println!("cargo:rustc-link-lib=android");  // Android system library

        // Architecture-specific settings
        match target_arch.as_str() {
            "aarch64" => {
                println!("cargo:rustc-link-arg=-Wl,--no-rosegment");
            }
            "arm" => {
                println!("cargo:rustc-link-arg=-Wl,--no-warn-mismatch");
            }
            _ => {}
        }

        println!("cargo:warning=Building for Android target: {} ({})", target_os, target_arch);
    } else {
        println!("cargo:warning=Building for non-Android target: {} ({})", target_os, target_arch);
        println!("cargo:warning=JNI functions will be available but may not work correctly outside Android");
    }

    // Verify that we're building as a cdylib for Android
    if target_os == "android" {
        let crate_type = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
        if !crate_type.contains("cdylib") {
            println!("cargo:warning=Building Android bridge without cdylib. JNI may not work correctly.");
        }
    }
}