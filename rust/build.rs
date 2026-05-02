use std::env;
use std::path::PathBuf;

fn main() {
    // Resolve lib/windows-x64/overdrive.dll relative to this Cargo.toml
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir  = manifest.parent().unwrap().join("lib").join("windows-x64");

    // Copy overdrive.dll to target/debug/ and target/debug/deps/ so tests find it
    let dll_src = lib_dir.join("overdrive.dll");
    if dll_src.exists() {
        let out = PathBuf::from(env::var("OUT_DIR").unwrap());
        // target/debug/deps/
        let deps = out.ancestors().nth(2).unwrap();
        // target/debug/
        let debug = deps.parent().unwrap();

        for dst in &[deps.join("overdrive.dll"), debug.join("overdrive.dll")] {
            std::fs::copy(&dll_src, dst).ok();
        }
        println!("cargo:warning=Copied overdrive.dll → {}", debug.display());
    } else {
        println!("cargo:warning=overdrive.dll not found at {}", dll_src.display());
    }
    println!("cargo:rerun-if-changed=../lib/windows-x64/overdrive.dll");
}
