use std::env;
use std::path::PathBuf;

// BEFORE: this script only ever looked at `lib/windows-x64/overdrive.dll`
// and copied *that* into target/debug — hardcoded, regardless of which OS
// the build was actually running on. On Linux/macOS the `if dll_src.exists()`
// check always failed, so `cargo test` silently never staged a native lib
// next to the test binary from build.rs (tests still passed only because
// `ffi::load()` independently walks `lib/{os}-{arch}/` at runtime — so this
// bug was masked, but the script's own stated purpose ("copy overdrive.dll
// to target/debug so tests find it") never worked outside Windows, and the
// misleading "cargo:warning=overdrive.dll not found" fired on every
// non-Windows build).
//
// AFTER: resolve the native lib name/dir for whatever TARGET the build is
// actually compiling for (cross-compilation aware, via the TARGET env var
// Cargo sets), and copy that one.
fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_root = manifest.parent().unwrap_or(&manifest).join("lib");

    let target = env::var("TARGET").unwrap_or_default();
    let (os_dir, lib_file) = target_lib_info(&target);

    let lib_dir = lib_root.join(os_dir);
    let dll_src = lib_dir.join(lib_file);

    if dll_src.exists() {
        let out = PathBuf::from(env::var("OUT_DIR").unwrap());
        // target/<profile>/deps/
        let deps = out.ancestors().nth(2).unwrap();
        // target/<profile>/
        let profile_dir = deps.parent().unwrap();

        for dst in &[deps.join(lib_file), profile_dir.join(lib_file)] {
            if let Err(e) = std::fs::copy(&dll_src, dst) {
                println!("cargo:warning=Failed to copy {} -> {}: {}", dll_src.display(), dst.display(), e);
            }
        }
        println!("cargo:warning=Copied {} -> {}", lib_file, profile_dir.display());
    } else {
        println!("cargo:warning={} not found at {}", lib_file, dll_src.display());
    }

    println!("cargo:rerun-if-changed={}", lib_dir.display());
    println!("cargo:rerun-if-env-changed=TARGET");
}

/// Map a Cargo `TARGET` triple to (bundled-lib-subdir, lib-filename).
/// Falls back to the host platform if `TARGET` isn't set (e.g. when the
/// build script itself is invoked outside of `cargo build`).
fn target_lib_info(target: &str) -> (&'static str, &'static str) {
    let is_windows = target.contains("windows") || (target.is_empty() && cfg!(target_os = "windows"));
    let is_macos   = target.contains("apple")   || (target.is_empty() && cfg!(target_os = "macos"));
    let is_arm64   = target.contains("aarch64") || (target.is_empty() && cfg!(target_arch = "aarch64"));

    if is_windows {
        ("windows-x64", "overdrive.dll")
    } else if is_macos {
        if is_arm64 { ("macos-arm64", "liboverdrive.dylib") } else { ("macos-x64", "liboverdrive.dylib") }
    } else if is_arm64 {
        ("linux-arm64", "liboverdrive.so")
    } else {
        ("linux-x64", "liboverdrive.so")
    }
}
