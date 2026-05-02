# Build overdrive.dll from OverDrive-DB server source (Windows)
$SERVER = (Resolve-Path "$PSScriptRoot\..\..\OverDrive-DB").Path
$OUT    = "$PSScriptRoot\..\lib\windows-x64\overdrive.dll"

Write-Host "Building from: $SERVER"
Set-Location $SERVER
cargo build --features ffi --release

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed"; exit 1
}

New-Item -ItemType Directory -Force (Split-Path $OUT) | Out-Null
Copy-Item "target\release\overdrive_db.dll" $OUT -Force
$kb = [math]::Round((Get-Item $OUT).Length / 1KB)
Write-Host "✅ overdrive.dll → $OUT ($($kb)KB)"
