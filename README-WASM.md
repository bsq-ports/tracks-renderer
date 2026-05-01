Bevy WASM build for Tauri frontend

This repo contains a `tracks-renderer` crate that can be built as native or as WASM for web embedding.

Overview
- Native: run `tracks-renderer` binary (desktop) — it uses Bevy and opens a native window.
- Web (WASM): build `tracks-renderer` with the `wasm` feature and generate JS bindings using `wasm-bindgen`. The generated JS package should be placed under `src/bevy_wasm_pkg` so `BevyWasmViewer` can import it.

Prerequisites
- Rust toolchain
- `wasm-bindgen-cli` (install via `cargo install -f wasm-bindgen-cli`)
- `rustup target add wasm32-unknown-unknown`

Automated script (from repo root)

This project includes an npm script that builds the wasm artifact and runs `wasm-bindgen`:

```bash
# from project root
pnpm run build:wasm
# or
npm run build:wasm
# or
yarn build:wasm
```

What the script does
1. Adds the `wasm32-unknown-unknown` target.
2. Builds `tracks-renderer` for that target with the `wasm` feature enabled.
3. Runs `wasm-bindgen` to generate a JS package under `src/bevy_wasm_pkg`.

Manual steps (if you prefer)

```bash
# add wasm target
rustup target add wasm32-unknown-unknown
# build the crate with the wasm feature
cargo build --target wasm32-unknown-unknown --release -p tracks-renderer --features wasm
# run wasm-bindgen (ensure wasm-bindgen-cli is installed)
wasm-bindgen target/wasm32-unknown-unknown/release/tracks_renderer.wasm --out-dir src/bevy_wasm_pkg --target web
```

After building
- The generated JS/WASM files will be in `src/bevy_wasm_pkg`.
- `BevyWasmViewer` imports the package (adjust path in `src/components/BevyWasmViewer.tsx` if needed) and calls `init('bevy-canvas')` on load.

Notes
- Building Bevy for wasm is non-trivial and may require additional plugin configuration or different Bevy/web plugin versions. If build errors occur, share the error output and I can adapt the crate setup (e.g., plugin selection, feature flags).
- The npm script assumes a Linux/macOS environment with `bash` available. Modify if using Windows.
