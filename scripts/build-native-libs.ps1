<#
.SYNOPSIS
    Build native libraries for all platforms.

.DESCRIPTION
    Builds the OverDrive-DB native library (cdylib) for all supported platforms:
    - Windows x64 (.dll)
    - Linux x64 (.so)
    - Linux ARM64 (.so)
    - macOS x64 (.dylib)
    - macOS ARM64 (.dylib)

.PARAMETER Platform
    Specific platform to build. Default: all.
    Options: windows-x64, linux-x64, linux-arm64, macos-x64, macos-arm64, all

.PARAMETER OutputDir
    Output directory for built libraries. Default: dist/native-libs/<version>

.EXAMPLE
    .\scripts\build-native-libs.ps1 -Platform all
    .\scripts\build-native-libs.ps1 -Platform windows-x64
#>

param(
    [ValidateSet("windows-x64", "linux-x64", "linux-arm64", "macos-x64", "macos-arm64", "all")]
    [string]$Platform = "all",
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Stop"

# Read version from Cargo.toml
$cargoToml = Get-Content "Cargo.toml" -Raw
if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
    $version = $Matches[1]
} else {
    $version = "0.0.0"
}

if (-not $OutputDir) {
    $OutputDir = "dist/native-libs/$version"
}

Write-Host "╔══════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  OverDrive-DB Native Library Builder v$version       ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Ensure output directory exists
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

function Build-Windows-x64 {
    Write-Host "[BUILD] Windows x64..." -ForegroundColor Yellow
    cargo build --release --lib --features ffi 2>&1
    if ($LASTEXITCODE -ne 0) { throw "Windows x64 build failed" }

    $src = "target/release/overdrive_db.dll"
    $dst = "$OutputDir/overdrive.dll"
    if (Test-Path $src) {
        Copy-Item $src $dst -Force
        $size = (Get-Item $dst).Length / 1MB
        Write-Host "  ✓ overdrive.dll ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
    } else {
        Write-Host "  ✗ overdrive_db.dll not found" -ForegroundColor Red
    }
}

function Build-Linux-x64 {
    Write-Host "[BUILD] Linux x64 (cross-compile)..." -ForegroundColor Yellow
    & cross build --release --lib --target x86_64-unknown-linux-gnu --features ffi 2>&1
    if ($LASTEXITCODE -ne 0) { throw "Linux x64 build failed" }

    $src = "target/x86_64-unknown-linux-gnu/release/liboverdrive_db.so"
    $dst = "$OutputDir/liboverdrive.so"
    if (Test-Path $src) {
        Copy-Item $src $dst -Force
        $size = (Get-Item $dst).Length / 1MB
        Write-Host "  ✓ liboverdrive.so ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
    }
}

function Build-Linux-ARM64 {
    Write-Host "[BUILD] Linux ARM64 (cross-compile)..." -ForegroundColor Yellow
    & cross build --release --lib --target aarch64-unknown-linux-gnu --features ffi 2>&1
    if ($LASTEXITCODE -ne 0) { throw "Linux ARM64 build failed" }

    $src = "target/aarch64-unknown-linux-gnu/release/liboverdrive_db.so"
    $dst = "$OutputDir/liboverdrive-arm64.so"
    if (Test-Path $src) {
        Copy-Item $src $dst -Force
        $size = (Get-Item $dst).Length / 1MB
        Write-Host "  ✓ liboverdrive-arm64.so ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
    }
}

function Build-macOS-x64 {
    Write-Host "[BUILD] macOS x64 (cross-compile)..." -ForegroundColor Yellow
    & cross build --release --lib --target x86_64-apple-darwin --features ffi 2>&1
    if ($LASTEXITCODE -ne 0) { throw "macOS x64 build failed" }

    $src = "target/x86_64-apple-darwin/release/liboverdrive_db.dylib"
    $dst = "$OutputDir/liboverdrive.dylib"
    if (Test-Path $src) {
        Copy-Item $src $dst -Force
        $size = (Get-Item $dst).Length / 1MB
        Write-Host "  ✓ liboverdrive.dylib ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
    }
}

function Build-macOS-ARM64 {
    Write-Host "[BUILD] macOS ARM64 (cross-compile)..." -ForegroundColor Yellow
    & cross build --release --lib --target aarch64-apple-darwin --features ffi 2>&1
    if ($LASTEXITCODE -ne 0) { throw "macOS ARM64 build failed" }

    $src = "target/aarch64-apple-darwin/release/liboverdrive_db.dylib"
    $dst = "$OutputDir/liboverdrive-arm64.dylib"
    if (Test-Path $src) {
        Copy-Item $src $dst -Force
        $size = (Get-Item $dst).Length / 1MB
        Write-Host "  ✓ liboverdrive-arm64.dylib ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
    }
}

function Copy-To-SDKs {
    Write-Host ""
    Write-Host "[DISTRIBUTE] Copying native libraries to SDK packages..." -ForegroundColor Yellow

    $sdkDirs = @{
        "IncodeSDK/nodejs/lib" = @("overdrive.dll", "liboverdrive.so", "liboverdrive-arm64.so", "liboverdrive.dylib", "liboverdrive-arm64.dylib")
        "IncodeSDK/python/overdrive" = @("overdrive.dll", "liboverdrive.so", "liboverdrive-arm64.so", "liboverdrive.dylib", "liboverdrive-arm64.dylib")
        "IncodeSDK/java/lib" = @("overdrive.dll", "liboverdrive.so", "liboverdrive-arm64.so", "liboverdrive.dylib", "liboverdrive-arm64.dylib")
        "IncodeSDK/go/lib" = @("overdrive.dll", "liboverdrive.so", "liboverdrive-arm64.so", "liboverdrive.dylib", "liboverdrive-arm64.dylib")
    }

    foreach ($dir in $sdkDirs.Keys) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        foreach ($lib in $sdkDirs[$dir]) {
            $src = "$OutputDir/$lib"
            if (Test-Path $src) {
                Copy-Item $src "$dir/$lib" -Force
                Write-Host "  ✓ $lib → $dir" -ForegroundColor Green
            }
        }
    }
}

# Execute builds
switch ($Platform) {
    "windows-x64" { Build-Windows-x64 }
    "linux-x64" { Build-Linux-x64 }
    "linux-arm64" { Build-Linux-ARM64 }
    "macos-x64" { Build-macOS-x64 }
    "macos-arm64" { Build-macOS-ARM64 }
    "all" {
        Build-Windows-x64
        try { Build-Linux-x64 } catch { Write-Host "  ⚠ Linux x64 skipped (cross not available)" -ForegroundColor DarkYellow }
        try { Build-Linux-ARM64 } catch { Write-Host "  ⚠ Linux ARM64 skipped (cross not available)" -ForegroundColor DarkYellow }
        try { Build-macOS-x64 } catch { Write-Host "  ⚠ macOS x64 skipped (cross not available)" -ForegroundColor DarkYellow }
        try { Build-macOS-ARM64 } catch { Write-Host "  ⚠ macOS ARM64 skipped (cross not available)" -ForegroundColor DarkYellow }
        Copy-To-SDKs
    }
}

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║  Build complete! Output: $OutputDir              ║" -ForegroundColor Green
Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Green

# Verify binary sizes
Write-Host ""
Write-Host "[VERIFY] Binary sizes:" -ForegroundColor Yellow
Get-ChildItem $OutputDir -File | ForEach-Object {
    $sizeMB = $_.Length / 1MB
    $status = if ($sizeMB -lt 20) { "✓" } else { "⚠ WARNING: >20MB" }
    Write-Host "  $status $($_.Name): $([math]::Round($sizeMB, 1)) MB" -ForegroundColor $(if ($sizeMB -lt 20) { "Green" } else { "Red" })
}
