# Yawnoc

A small Tauri desktop app that displays a native Rust 16x16 Conway Game of Life board.

The UI follows the Device Envoy Conway WASM demo, but the Game of Life state and controls live in the Tauri Rust backend rather than WebAssembly.

The checked-in `dist` directory is intentionally just static frontend assets for
Tauri to embed. Keep it in sync with `index.html`, `src`, and `public` when those
files change.

## Run

```sh
cargo app
```

or, explicitly:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --target-dir target/app
```

On Linux, Tauri needs the WebKitGTK/GTK development packages available to
`pkg-config`. On Debian/Ubuntu-like systems this is typically:

```sh
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```
