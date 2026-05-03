# SHA-256 Checksum Verification Script for Native Libraries
# Usage: .\verify-checksums.ps1

Write-Host "OverDrive-DB Native Libraries - SHA-256 Verification" -ForegroundColor Green
Write-Host "=====================================================" -ForegroundColor Green
Write-Host ""

# Expected checksums from v1.4.0 release
$expectedChecksums = @{
    "overdrive.dll" = "41A963CF20D9DD718CC4C09BCC10F9EE5D7079FB8FF76EB4CECC56AFDEE4F3C9"
    "liboverdrive-linux-x64.so" = "AC28959902E52F9F33CD54D31C261F79F59AF4B6D03927D06902C4E1ADC9736B"
    "liboverdrive-linux-arm64.so" = "66F8AAD50AC37D6F6FDB78DDF9AFEF4DEB70787DC564124A067577AD3C022DDB"
    "liboverdrive-macos-x64.dylib" = "B19283ABDB53633EEBB81F796B85264D40BBF3275DBCAEC9BE81DDD524F75169"
    "liboverdrive-macos-arm64.dylib" = "C4AA4332BD182189A761B517F8AE04A594B8E5BB344C8327D6EE4ACF7414EF2A"
}

$allValid = $true
$libPath = Split-Path -Parent $MyInvocation.MyCommand.Path

foreach ($library in $expectedChecksums.Keys) {
    $filePath = Join-Path $libPath $library
    
    if (Test-Path $filePath) {
        Write-Host "Verifying: $library" -ForegroundColor Yellow
        
        $actualHash = (Get-FileHash $filePath -Algorithm SHA256).Hash
        $expectedHash = $expectedChecksums[$library]
        
        if ($actualHash -eq $expectedHash) {
            Write-Host "  ✅ VALID - Checksum matches" -ForegroundColor Green
        } else {
            Write-Host "  ❌ INVALID - Checksum mismatch!" -ForegroundColor Red
            Write-Host "    Expected: $expectedHash" -ForegroundColor Red
            Write-Host "    Actual:   $actualHash" -ForegroundColor Red
            $allValid = $false
        }
    } else {
        Write-Host "  ❌ MISSING - File not found: $library" -ForegroundColor Red
        $allValid = $false
    }
    Write-Host ""
}

Write-Host "=====================================================" -ForegroundColor Green
if ($allValid) {
    Write-Host "✅ ALL LIBRARIES VERIFIED SUCCESSFULLY" -ForegroundColor Green
    Write-Host "All native libraries have valid SHA-256 checksums." -ForegroundColor Green
} else {
    Write-Host "❌ VERIFICATION FAILED" -ForegroundColor Red
    Write-Host "One or more libraries failed checksum verification." -ForegroundColor Red
    Write-Host "Please re-download libraries from the official v1.4.0 release." -ForegroundColor Red
}
Write-Host "=====================================================" -ForegroundColor Green