#!/usr/bin/env bash
# WEBREL1-E2: build the engine's web example (engine/crates/aegis_ui/examples/web_min.rs) for wasm32, bind it with
# wasm-bindgen-cli 0.2.129 (it must match the locked wasm-bindgen exactly) and check it in headless Chromium
# (tools/web/web_min_check.js: a frame is presented, key, click and right click reach the screen, at device pixel
# ratios 1 and 2). Not part of the gate: it needs the wasm32 target, wasm-bindgen-cli, Node with Playwright and Chromium.
# Usage: tools/web/web_min_check.sh [served folder]   (default: a new temporary folder)
# Build output: $CARGO_TARGET_DIR_WASM, default target-wasm/ (kept apart from the native build).
# Install the binder once: cargo install wasm-bindgen-cli --version 0.2.129 --locked
set -euo pipefail
cd "$(dirname "$0")/../.." || exit 2
target=${CARGO_TARGET_DIR_WASM:-$PWD/target-wasm}
out=${1:-$(mktemp -d)}
if ! wasm-bindgen --version 2>/dev/null | grep -q " 0.2.129$"; then
  echo "web_min: needs wasm-bindgen-cli 0.2.129 (cargo install wasm-bindgen-cli --version 0.2.129 --locked)"
  exit 2
fi
CARGO_TARGET_DIR=$target cargo build --release --target wasm32-unknown-unknown -p aegis_ui --example web_min
mkdir -p "$out"
wasm-bindgen --target web --no-typescript --out-dir "$out" --out-name web_min "$target/wasm32-unknown-unknown/release/examples/web_min.wasm"
node tools/web/web_min_check.js "$out"
