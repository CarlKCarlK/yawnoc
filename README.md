# Yawnoc

A small Tauri desktop app that displays a native Rust 16x16 Conway Game of Life board.

The UI follows the Device Envoy Conway WASM demo, but the Game of Life state and controls live in the Tauri Rust backend rather than WebAssembly.

The checked-in `dist` directory is intentionally just static frontend assets for
Tauri to embed. Keep it in sync with `index.html`, `src`, and `public` when those
files change.

## Run

```sh
just run
```

or, explicitly:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --target-dir target/app
```

## Build Windows Installer (Local)

From Windows in this repo:

```sh
just win-installer
```

The installer `.exe` is written under:

`target/app/release/bundle/nsis/`

The recipe auto-generates icon assets and runs the installer build script.
Make sure the running app is closed before building.

## Run In Browser (WASM)

Build WASM package and web output:

```sh
just wasm-build
```

Serve locally:

```sh
just wasm-serve
```

Then open:

`http://localhost:4173`

Notes:
- The same UI now auto-selects backend:
  - Tauri commands when running in desktop app.
  - WASM module in a Web Worker when running in browser.
- `prev` SAT predecessor search runs in browser mode too, executed inside the worker so the UI thread stays responsive.

## Build Installers In CI

The `Release` GitHub Actions workflow builds installers for Linux, Windows, and macOS.

To trigger a full release with assets attached:

```sh
just release-tag 0.1.1
```

After CI completes, installers are attached to the GitHub Release for that tag.

You can also run `Release` manually (`workflow_dispatch`) to get CI artifacts without publishing a tag release.

On Linux, Tauri needs the WebKitGTK/GTK development packages available to
`pkg-config`. On Debian/Ubuntu-like systems this is typically:

```sh
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```
