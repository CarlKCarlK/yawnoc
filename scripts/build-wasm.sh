#!/usr/bin/env bash
set -euo pipefail

# Ensure wasm target is available for Rust.
rustup target add wasm32-unknown-unknown

# Ensure wasm-pack is available.
if ! command -v wasm-pack &>/dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack --locked
fi

wasm-pack build wasm --target web --out-dir pkg --out-name yawnoc_wasm --release

mkdir -p dist-wasm/src dist-wasm/public dist-wasm/wasm

cp index.html dist-wasm/index.html
cp -r src/. dist-wasm/src/
cp -r public/. dist-wasm/public/
cp public/sw.js dist-wasm/sw.js
cp -r wasm/pkg dist-wasm/wasm/

# Copy app icon into dist-wasm/public/ for PWA manifest.
cp src-tauri/icons/icon.png dist-wasm/public/icon.png

echo "WASM web output ready in dist-wasm/."
