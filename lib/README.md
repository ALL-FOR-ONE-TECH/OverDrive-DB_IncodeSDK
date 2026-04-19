# lib/ — Native Library Directory

This directory contains the prebuilt OverDrive native shared library.

## Files

| File | Platform |
|------|----------|
| `overdrive.dll` | Windows x64 |
| `liboverdrive.so` | Linux x64 / ARM64 |
| `liboverdrive.dylib` | macOS x64 / ARM64 |

## How it works

When you run `cargo build` or `cargo run`, the `build.rs` script automatically copies the native library
from this directory to your `target/` directory so the executable can find it at runtime.

The `src/dynamic.rs` loader searches these directories at startup (in order):
1. Current working directory
2. `./lib/` subdirectory  
3. Executable directory
4. Executable's `./lib/` subdirectory
5. If not found → auto-downloads from GitHub Releases

## Manual placement

If you downloaded a binary from GitHub Releases, place it here:

```
your-project/
├── Cargo.toml
├── src/
└── lib/
    └── overdrive.dll   ← place here
```
