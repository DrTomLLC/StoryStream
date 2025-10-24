// Build script for storystream-android-bridge
//
// Emits warnings when building for non-Android platforms

fn main() {
    // Emit warnings for non-Android builds
    #[cfg(not(target_os = "android"))]
    {
        println!("cargo:warning=Building for non-Android target");
        println!("cargo:warning=JNI functions will be available but may not work correctly outside Android");
    }

    // Tell cargo to rerun if target changes
    println!("cargo:rerun-if-env-changed=TARGET");
}
