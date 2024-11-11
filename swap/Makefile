PINNED_TOOLCHAIN := $(shell cat rust-toolchain)

prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rustfmt --toolchain ${PINNED_TOOLCHAIN}

.PHONY:	build-contract
build-contract:
	cargo build --release --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/swap.wasm

setup-test: build-contract
	mkdir -p tests/wasm
	cp ./target/wasm32-unknown-unknown/release/swap.wasm tests/wasm

test: setup-test
	cd tests && cargo test

clippy:
    cargo clippy --all-targets -- -D warnings

check-lint: clippy
    cargo fmt -- --check

lint: clippy
	cargo fmt

clean:
	cargo clean
	rm -rf tests/wasm
