#!/usr/bin/env bash
# Rebuild the WebAssembly interpreter used by the static playground in playground/.
set -euo pipefail
cd "$(dirname "$0")/../adarsh-rs"
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
cargo rustc --release --lib --target wasm32-unknown-unknown --crate-type cdylib
cp target/wasm32-unknown-unknown/release/adarshlang.wasm ../playground/adarshlang.wasm
echo "playground/adarshlang.wasm updated ($(du -h ../playground/adarshlang.wasm | cut -f1))"
