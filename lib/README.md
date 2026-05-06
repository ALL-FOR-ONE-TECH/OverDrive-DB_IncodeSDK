# Native Binaries — OverDrive-DB v2.3.0

Pre-built by GitHub Actions CI. Do not edit these files manually.

| Platform | File | CI Status |
|---|---|---|
| 🪟 Windows x64 | `windows-x64/overdrive.dll` | ✅ |
| 🐧 Linux x64 | `linux-x64/liboverdrive.so` | ✅ |
| 🐧 Linux ARM64 | `linux-arm64/liboverdrive.so` | ✅ |
| 🍎 macOS x64 (Intel) | `macos-x64/liboverdrive.dylib` | ✅ |
| 🍎 macOS ARM64 (Apple Silicon) | `macos-arm64/liboverdrive.dylib` | ✅ |

See [CHECKSUMS.md](CHECKSUMS.md) for SHA-256 hashes.

## Override (custom build)

```bash
# Use your own binary instead of the bundled one
export OVERDRIVE_LIB_PATH=/path/to/liboverdrive.so
```

## Rebuild locally

```powershell
# Windows
cargo build --release --features ffi
Copy-Item target\release\overdrive_db.dll IncodeSDK\lib\windows-x64\overdrive.dll
```

```bash
# Linux / macOS
cargo build --release --features ffi
cp target/release/liboverdrive_db.so    IncodeSDK/lib/linux-x64/liboverdrive.so
cp target/release/liboverdrive_db.dylib IncodeSDK/lib/macos-arm64/liboverdrive.dylib
```

## Source

Built from [karthikeyanV2K/OverDrive-DB](https://github.com/karthikeyanV2K/OverDrive-DB)
