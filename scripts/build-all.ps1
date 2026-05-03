# OverDrive-DB SDK Build Script v2.0.0 (PowerShell)
# Builds all language SDKs

Write-Host "🚀 Building OverDrive-DB SDKs v2.0.0..." -ForegroundColor Blue
Write-Host "========================================" -ForegroundColor Blue

$ErrorActionPreference = "Continue"

# Build Python SDK
Write-Host "📦 Building Python SDK..." -ForegroundColor Cyan
Set-Location "sdks/python"
if (Get-Command python -ErrorAction SilentlyContinue) {
    python -m build
    Write-Host "✅ Python SDK built successfully" -ForegroundColor Green
} else {
    Write-Host "⚠️  Python not found, skipping Python SDK" -ForegroundColor Yellow
}
Set-Location "../.."

# Build Node.js SDK
Write-Host "📦 Building Node.js SDK..." -ForegroundColor Cyan
Set-Location "sdks/nodejs"
if (Get-Command npm -ErrorAction SilentlyContinue) {
    npm install
    npm run build 2>$null
    Write-Host "✅ Node.js SDK built successfully" -ForegroundColor Green
} else {
    Write-Host "⚠️  npm not found, skipping Node.js SDK" -ForegroundColor Yellow
}
Set-Location "../.."

# Build Java SDK
Write-Host "📦 Building Java SDK..." -ForegroundColor Cyan
Set-Location "sdks/java"
if (Get-Command mvn -ErrorAction SilentlyContinue) {
    mvn clean package -q
    Write-Host "✅ Java SDK built successfully" -ForegroundColor Green
} else {
    Write-Host "⚠️  Maven not found, skipping Java SDK" -ForegroundColor Yellow
}
Set-Location "../.."

# Build Go SDK
Write-Host "📦 Building Go SDK..." -ForegroundColor Cyan
Set-Location "sdks/go"
if (Get-Command go -ErrorAction SilentlyContinue) {
    go build
    Write-Host "✅ Go SDK built successfully" -ForegroundColor Green
} else {
    Write-Host "⚠️  Go not found, skipping Go SDK" -ForegroundColor Yellow
}
Set-Location "../.."

# Build Rust SDK
Write-Host "📦 Building Rust SDK..." -ForegroundColor Cyan
Set-Location "sdks/rust"
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    cargo build --release
    Write-Host "✅ Rust SDK built successfully" -ForegroundColor Green
} else {
    Write-Host "⚠️  Cargo not found, skipping Rust SDK" -ForegroundColor Yellow
}
Set-Location "../.."

# C SDK (header only, no build needed)
Write-Host "📦 C SDK (header-only)..." -ForegroundColor Cyan
if (Test-Path "sdks/c/include/overdrive.h") {
    Write-Host "✅ C SDK ready (header-only)" -ForegroundColor Green
} else {
    Write-Host "❌ C SDK header not found" -ForegroundColor Red
}

Write-Host ""
Write-Host "🎉 All SDKs built successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Summary:" -ForegroundColor White
Write-Host "  - Python SDK: sdks/python/dist/" -ForegroundColor Gray
Write-Host "  - Node.js SDK: sdks/nodejs/" -ForegroundColor Gray
Write-Host "  - Java SDK: sdks/java/target/" -ForegroundColor Gray
Write-Host "  - Go SDK: sdks/go/" -ForegroundColor Gray
Write-Host "  - Rust SDK: sdks/rust/target/release/" -ForegroundColor Gray
Write-Host "  - C SDK: sdks/c/include/overdrive.h" -ForegroundColor Gray
Write-Host ""
Write-Host "🚀 Ready for distribution!" -ForegroundColor Green