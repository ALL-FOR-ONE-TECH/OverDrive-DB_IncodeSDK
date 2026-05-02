# Native Binaries — OverDrive-DB v2.2.0

Auto-built by GitHub Actions from server source.

| Platform | File | Built by CI |
|---|---|---|
| Windows x64 | `windows-x64/overdrive.dll` | ✅ |
| Linux x64 | `linux-x64/liboverdrive.so` | ✅ |
| Linux ARM64 | `linux-arm64/liboverdrive.so` | ✅ |
| macOS x64 | `macos-x64/liboverdrive.dylib` | ✅ |
| macOS ARM64 | `macos-arm64/liboverdrive.dylib` | ✅ |

**Override:** Set `OVERDRIVE_LIB_PATH` env var to load a custom binary.

**Rebuild locally (Windows):**
```powershell
.\scripts\build-native.ps1
```

**Rebuild locally (Linux/macOS):**
```bash
chmod +x scripts/build-native.sh && ./scripts/build-native.sh
```
