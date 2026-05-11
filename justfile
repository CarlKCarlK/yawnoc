set shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

default:
  just --list

run:
  cargo app

icons:
  cargo tauri icon src-tauri/icons/icon.png

win-installer: icons
  powershell -ExecutionPolicy Bypass -File .\scripts\build-windows-installer.ps1

wasm-build:
  powershell -ExecutionPolicy Bypass -File .\scripts\build-wasm.ps1

# Use on Linux / macOS / CI
wasm-build-unix:
  bash scripts/build-wasm.sh

wasm-serve:
  python -m http.server 4173 --directory dist-wasm

release-tag version:
  git tag v{{version}}
  git push origin v{{version}}
