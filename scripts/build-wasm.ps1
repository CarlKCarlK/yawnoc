Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Ensure wasm target is available for Rust.
rustup target add wasm32-unknown-unknown
if ($LASTEXITCODE -ne 0) {
    throw "Failed to add wasm32-unknown-unknown target."
}

# Ensure wasm-pack is available.
$hasWasmPack = $null -ne (Get-Command wasm-pack -ErrorAction SilentlyContinue)
if (-not $hasWasmPack) {
    Write-Host "Installing wasm-pack..."
    cargo install wasm-pack --locked
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to install wasm-pack."
    }
}

wasm-pack build wasm --target web --out-dir pkg --out-name yawnoc_wasm --release
if ($LASTEXITCODE -ne 0) {
    throw "wasm-pack build failed."
}

New-Item -ItemType Directory -Force dist-wasm | Out-Null
New-Item -ItemType Directory -Force dist-wasm/src | Out-Null
New-Item -ItemType Directory -Force dist-wasm/public | Out-Null
New-Item -ItemType Directory -Force dist-wasm/wasm | Out-Null
Copy-Item index.html dist-wasm/index.html -Force
Copy-Item src/* dist-wasm/src -Recurse -Force
Copy-Item public/* dist-wasm/public -Recurse -Force
Copy-Item wasm/pkg dist-wasm/wasm -Recurse -Force

# Copy app icon into dist-wasm/public/ for PWA manifest.
Copy-Item src-tauri/icons/icon.png dist-wasm/public/icon.png -Force

Write-Host "WASM web output ready in dist-wasm/."
