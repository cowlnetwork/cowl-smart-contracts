#!/bin/bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
wasm-strip target/wasm32-unknown-unknown/release/cowl_ghost_swap.wasm 2>/dev/null || true
