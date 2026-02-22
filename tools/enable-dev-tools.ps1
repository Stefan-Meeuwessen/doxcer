# ==========================================
# Dev Environment Unlock Script
# - Sets Execution Policy to Unrestricted
# - Allows Rust (cargo + rustc)
# - Allows rustup toolchains
# - Allows Git Bash
# - Allows Doxcer managed executable
# ==========================================

Write-Host "=== Setting Execution Policy ===" -ForegroundColor Cyan

try {
    Set-ExecutionPolicy -ExecutionPolicy Unrestricted -Scope LocalMachine -Force
    Write-Host "Execution policy set to Unrestricted (LocalMachine)" -ForegroundColor Green
} catch {
    Write-Host "Failed to set LocalMachine scope. Trying CurrentUser..." -ForegroundColor Yellow
    Set-ExecutionPolicy -ExecutionPolicy Unrestricted -Scope CurrentUser -Force
}

Write-Host "`n=== Adding Controlled Folder Access Exceptions ===" -ForegroundColor Cyan

function Add-CFAException($path) {
    if (Test-Path $path) {
        Add-MpPreference -ControlledFolderAccessAllowedApplications $path -ErrorAction SilentlyContinue
        Write-Host "Allowed: $path" -ForegroundColor Green
    } else {
        Write-Host "Skipped (not found): $path" -ForegroundColor DarkYellow
    }
}

# Rust (cargo default)
Add-CFAException "$env:USERPROFILE\.cargo\bin\cargo.exe"
Add-CFAException "$env:USERPROFILE\.cargo\bin\rustc.exe"

# rustup toolchains (all installed toolchains)
$toolchainsPath = "$env:USERPROFILE\.rustup\toolchains"
if (Test-Path $toolchainsPath) {
    Get-ChildItem $toolchainsPath -Recurse -Filter cargo.exe -ErrorAction SilentlyContinue | ForEach-Object {
        Add-CFAException $_.FullName
    }

    Get-ChildItem $toolchainsPath -Recurse -Filter rustc.exe -ErrorAction SilentlyContinue | ForEach-Object {
        Add-CFAException $_.FullName
    }
}

# Git Bash
Add-CFAException "C:\Program Files\Git\bin\bash.exe"
Add-CFAException "C:\Program Files\Git\usr\bin\bash.exe"
Add-CFAException "C:\Program Files\Git\git-bash.exe"
Add-CFAException "C:\Program Files\Git\mingw64\bin\git.exe"

# Doxcer (managed install for current user)
Add-CFAException "$env:LOCALAPPDATA\doxcer\bin\doxcer.exe"

Write-Host "`n=== Done ===" -ForegroundColor Cyan
Write-Host "Verify with:" -ForegroundColor White
Write-Host "  Get-MpPreference | Select -Expand ControlledFolderAccessAllowedApplications"
