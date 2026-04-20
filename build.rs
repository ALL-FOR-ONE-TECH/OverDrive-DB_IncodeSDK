// build.rs — OverDrive InCode SDK (Rust)
//
// This build script:
// 1. Emits linker search paths so the native library is found at link time
//    when building from source (not needed for crates.io users).
// 2. Copies the native library from lib/ to the output directory so it is
//    found at runtime next to the compiled binary.
// 3. Emits cargo:rustc-link-search so `cargo test` and `cargo run` work
//    without manual PATH setup.

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=lib/");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let lib_dir = Path::new(&manifest_dir).join("lib");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_default());

    // Determine the native library name for this platform
    let (lib_file, _link_name) = if cfg!(target_os = "windows") {
        ("overdrive.dll", "overdrive")
    } else if cfg!(target_os = "macos") {
        ("liboverdrive.dylib", "overdrive")
    } else {
        ("liboverdrive.so", "overdrive")
    };

    // The SDK uses libloading for runtime dynamic loading — no link-time
    // linking against the native library is needed. The lib/ directory
    // is only used to copy the native library to the output directory
    // for convenience during development.

    // Copy the native library to the output directory so it's found at runtime
    // This helps `cargo run` and `cargo test` work without manual setup
    let lib_path = lib_dir.join(lib_file);
    if lib_path.exists() {
        // Copy to OUT_DIR/../../../ (the target/debug or target/release directory)
        // Walk up from OUT_DIR (which is target/{profile}/build/{pkg}/out)
        let mut target_dir = out_dir.clone();
        for _ in 0..3 {
            target_dir = match target_dir.parent() {
                Some(p) => p.to_path_buf(),
                None => break,
            };
        }

        let dest = target_dir.join(lib_file);
        if !dest.exists() {
            if let Err(e) = std::fs::copy(&lib_path, &dest) {
                eprintln!(
                    "cargo:warning=Could not copy {} to {}: {}",
                    lib_path.display(),
                    dest.display(),
                    e
                );
            } else {
                eprintln!(
                    "cargo:warning=Copied {} to {}",
                    lib_file,
                    dest.display()
                );
            }
        }
    } else {
        // Native library not found in lib/ — emit a helpful warning
        eprintln!(
            "cargo:warning=OverDrive native library not found at {}",
            lib_path.display()
        );
        eprintln!(
            "cargo:warning=Download {} from: https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest",
            lib_file
        );
        eprintln!(
            "cargo:warning=Place it in the lib/ directory next to Cargo.toml"
        );
    }

    // cbindgen header generation is handled separately via:
    //   cargo build --features generate-header
    // Requires adding cbindgen to [build-dependencies] in Cargo.toml first.
    // Not run in CI — skipped here intentionally.
}
