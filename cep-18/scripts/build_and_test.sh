#!/bin/bash

# Exit on error
set -e

echo "Building and testing COWL token contract..."

# Check if required tools are installed
command -v cargo >/dev/null 2>&1 || { echo "cargo is required but not installed. Aborting." >&2; exit 1; }
command -v wasm-strip >/dev/null 2>&1 || { echo "wasm-strip is required but not installed. Installing..." >&2; cargo install wasm-strip; }
command -v cargo-casper >/dev/null 2>&1 || { echo "cargo-casper is required but not installed. Installing..." >&2; cargo install cargo-casper; }

# Ensure wasm32 target is installed
rustup target add wasm32-unknown-unknown

# Clean previous builds
echo "Cleaning previous builds..."
make clean

# Build contract
echo "Building contract..."
make build

# Run tests
echo "Running tests..."
make test-with-logging

# Check if tests passed
if [ $? -eq 0 ]; then
    echo "✅ All tests passed!"
    echo "Contract WASM located at: target/wasm32-unknown-unknown/release/cowl_token.wasm"
else
    echo "❌ Tests failed!"
    exit 1
fi
