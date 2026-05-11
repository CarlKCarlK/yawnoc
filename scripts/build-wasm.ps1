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

$hasPython = $null -ne (Get-Command python -ErrorAction SilentlyContinue)
$hasPillow = $false
if ($hasPython) {
    python -c "from PIL import Image" 2>$null
    $hasPillow = $LASTEXITCODE -eq 0
}
if ($hasPython -and $hasPillow) {
    python scripts/gen-icon-16.py
    if ($LASTEXITCODE -ne 0) {
        throw "Icon generation failed."
    }
} else {
    Write-Warning "python/Pillow not found; using any existing small icons."
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
Copy-Item public/manifest.json dist-wasm/manifest.json -Force
Copy-Item src/* dist-wasm/src -Recurse -Force
Copy-Item public/* dist-wasm/public -Recurse -Force
Copy-Item public/sw.js dist-wasm/sw.js -Force
Copy-Item wasm/pkg dist-wasm/wasm -Recurse -Force

# Copy app icon into dist-wasm/public/ for PWA manifest.
Copy-Item src-tauri/icons/icon.png dist-wasm/public/icon.png -Force
if (Test-Path public/icon-16.png) { Copy-Item public/icon-16.png dist-wasm/public/icon-16.png -Force }
if (Test-Path public/icon-32.png) { Copy-Item public/icon-32.png dist-wasm/public/icon-32.png -Force }

Write-Host "WASM web output ready in dist-wasm/."
