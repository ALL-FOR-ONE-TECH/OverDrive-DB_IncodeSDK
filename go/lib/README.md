# OverDrive Native Library — Go SDK

Place the native library for your platform in this directory before building.

## Download

Download from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest):

| Platform | File to place here |
|----------|--------------------|
| Windows x64 | `overdrive.dll` + `liboverdrive.dll.a` |
| Linux x64 | `liboverdrive.so` |
| Linux ARM64 | `liboverdrive-arm64.so` → rename to `liboverdrive.so` |
| macOS x64 | `liboverdrive.dylib` |
| macOS ARM64 | `liboverdrive.dylib` |

## Windows — Generating the Import Library

The Go SDK uses CGo with MinGW on Windows. You need both `overdrive.dll` and
a MinGW-compatible import library `liboverdrive.dll.a`.

Generate it from the DLL:

```bash
# Using dlltool (comes with MinGW/MSYS2)
dlltool -d overdrive.def -l liboverdrive.dll.a

# Or generate the .def file first from the DLL:
gendef overdrive.dll
dlltool -d overdrive.def -l liboverdrive.dll.a
```

Or use the pre-generated `liboverdrive.dll.a` from the GitHub Release assets.

## Build

```bash
# Set CGO flags to find the library
export CGO_LDFLAGS="-L$(pwd)/lib"
export CGO_CFLAGS="-I$(pwd)"

go build ./...
go test ./...
```

## Runtime

On Windows, `overdrive.dll` must be in the same directory as your compiled binary,
or on the system PATH.

On Linux/macOS, set `LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH` to the `lib/` directory,
or install the library to `/usr/local/lib`.
