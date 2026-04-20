# Native Libraries v1.4.0 - Download Summary

## Overview
This directory contains all native libraries downloaded from the OverDrive-DB v1.4.0 release for cross-platform support.

## Downloaded Libraries

| Platform | File | Size | Status |
|----------|------|------|--------|
| Windows x64 | `overdrive.dll` | 3.3 MB | ✅ Available |
| Linux x64 | `liboverdrive-linux-x64.so` | 3.9 MB | ✅ Downloaded |
| Linux ARM64 | `liboverdrive-linux-arm64.so` | 3.7 MB | ✅ Downloaded |
| macOS x64 | `liboverdrive-macos-x64.dylib` | 3.6 MB | ✅ Downloaded |
| macOS ARM64 | `liboverdrive-macos-arm64.dylib` | 3.4 MB | ✅ Downloaded |

## Source
All libraries were downloaded from: https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/tag/v1.4.0

## Verification
- All platform-specific native libraries are present
- File sizes match the v1.4.0 release specifications
- ✅ **SHA-256 checksums verified** (see CHECKSUMS_SHA256.md for details)
- Libraries are ready for bundling in the JAR

## Security Checksums
SHA-256 checksums have been calculated and documented for all libraries:
- `overdrive.dll`: `41A963CF20D9DD718CC4C09BCC10F9EE5D7079FB8FF76EB4CECC56AFDEE4F3C9`
- `liboverdrive-linux-x64.so`: `AC28959902E52F9F33CD54D31C261F79F59AF4B6D03927D06902C4E1ADC9736B`
- `liboverdrive-linux-arm64.so`: `66F8AAD50AC37D6F6FDB78DDF9AFEF4DEB70787DC564124A067577AD3C022DDB`
- `liboverdrive-macos-x64.dylib`: `B19283ABDB53633EEBB81F796B85264D40BBF3275DBCAEC9BE81DDD524F75169`
- `liboverdrive-macos-arm64.dylib`: `C4AA4332BD182189A761B517F8AE04A594B8E5BB344C8327D6EE4ACF7414EF2A`

For complete verification details, see: `CHECKSUMS_SHA256.md`

## Next Steps
These libraries will be used in Task 2.1.3 to organize them in the `src/main/resources/native/` directory structure for cross-platform loading.

## Notes
- The existing `overdrive.dll` was already present (Windows x64)
- Added support for Linux x64, Linux ARM64, macOS x64, and macOS ARM64
- All libraries are compatible with the v1.4.0 OverDrive-DB release