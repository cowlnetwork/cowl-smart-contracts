prepare:
	rustup target add wasm32-unknown-unknown
	cargo install cargo-casper

build-contract:
	cargo build --release --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/cowl_token.wasm

test:
	cargo test

clippy:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all -- --check

check-lint: clippy fmt

build: check-lint build-contract

clean:
	cargo clean
	rm -rf target

.PHONY: prepare build-contract test clippy fmt check-lint build clean
