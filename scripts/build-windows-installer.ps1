Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$isWindowsHost = $false
if (Get-Variable -Name IsWindows -ErrorAction SilentlyContinue) {
    $isWindowsHost = [bool]$IsWindows
} else {
    $isWindowsHost = ($env:OS -eq "Windows_NT")
}

if (-not $isWindowsHost) {
    throw "This script is for Windows only."
}

if (Get-Process -Name "yawnoc" -ErrorAction SilentlyContinue) {
    throw "yawnoc.exe is running. Close the app and run the installer build again."
}

# Ensure cargo-tauri is available.
try {
    cargo tauri --version | Out-Null
} catch {
    Write-Host "Installing tauri-cli..."
    cargo install tauri-cli --locked
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to install tauri-cli."
    }
}

# Generate platform icon assets from the canonical source icon.
cargo tauri icon src-tauri/icons/icon.png
if ($LASTEXITCODE -ne 0) {
    throw "Failed to generate icon assets."
}

cargo tauri build --bundles nsis -- --target-dir ../target/app
if ($LASTEXITCODE -ne 0) {
    throw "Failed to build NSIS installer."
}

$installerDir = "target/app/release/bundle/nsis"
if (Test-Path $installerDir) {
    Write-Host "Installer output:"
    Get-ChildItem $installerDir -Filter *.exe | Select-Object FullName
}
