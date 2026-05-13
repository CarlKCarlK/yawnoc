# Leptos UI Spike

This folder is an experiment for replacing the JS frontend with Leptos.

## Purpose

- Validate Rust/WASM UI rendering with Leptos.
- Validate calling Tauri backend commands from Leptos.
- Keep current `src/main.js` path untouched while we evaluate tradeoffs.

## Run (standalone browser)

Requires `trunk` and target `wasm32-unknown-unknown`.

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd leptos-ui
trunk serve
```

When run outside Tauri, backend calls will report `tauri invoke unavailable`.

## Run inside Tauri (next step)

To fully evaluate migration, the next step is to point Tauri frontend dist to this app's bundle output and wire parity features (canvas board, key mapping, search progress UI).
