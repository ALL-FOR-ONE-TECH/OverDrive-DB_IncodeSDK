// build.rs — OverDrive InCode SDK (Rust)
//
// This build script:
// 1. Copies lib/overdrive.dll → target/{profile}/  (found by cargo run)
// 2. Copies lib/overdrive.dll → target/{profile}/deps/  (found by cargo test)
// 3. Always overwrites to stay in sync with the lib/ source of truth.

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=lib/");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let lib_dir = Path::new(&manifest_dir).join("lib");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_default());

    // Determine the native library name for this platform
    let lib_file = if cfg!(target_os = "windows") {
        "overdrive.dll"
    } else if cfg!(target_os = "macos") {
        "liboverdrive.dylib"
    } else {
        "liboverdrive.so"
    };

    // Source: IncodeSDK/lib/overdrive.dll (always present in repo)
    let lib_src = lib_dir.join(lib_file);

    if !lib_src.exists() {
        // Native library not in lib/ — emit a clear warning
        println!(
            "cargo:warning=OverDrive native library not found at {}",
            lib_src.display()
        );
        println!(
            "cargo:warning=Download {} from: https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest",
            lib_file
        );
        println!("cargo:warning=Place it in the lib/ directory next to Cargo.toml");
        return;
    }

    // OUT_DIR is: target/{profile}/build/overdrive-db-{hash}/out
    // Walk up 3 levels to reach: target/{profile}/
    let mut profile_dir = out_dir.clone();
    for _ in 0..3 {
        profile_dir = match profile_dir.parent() {
            Some(p) => p.to_path_buf(),
            None => {
                println!("cargo:warning=Could not resolve target profile directory");
                return;
            }
        };
    }

    // Copy destinations:
    //   target/{profile}/overdrive.dll         — found by cargo run + dynamic.rs exe_dir search
    //   target/{profile}/deps/overdrive.dll    — found by cargo test (test binary lives in deps/)
    let destinations = vec![
        profile_dir.join(lib_file),
        profile_dir.join("deps").join(lib_file),
    ];

    for dest in &destinations {
        // Create parent dir if needed (deps/ may not exist yet)
        if let Some(parent) = dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Always copy — keeps dest in sync with lib/ source of truth
        // (don't skip if file exists — stale files cause silent failures)
        match std::fs::copy(&lib_src, dest) {
            Ok(_) => {
                println!(
                    "cargo:warning=Copied {} → {}",
                    lib_file,
                    dest.display()
                );
            }
            Err(e) => {
                println!(
                    "cargo:warning=Could not copy {} to {}: {}",
                    lib_src.display(),
                    dest.display(),
                    e
                );
            }
        }
    }

    // cbindgen header generation is handled separately via:
    //   cargo build --features generate-header
    // Not run in CI — skipped here intentionally.
}
