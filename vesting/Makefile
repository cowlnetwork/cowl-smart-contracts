PINNED_TOOLCHAIN := $(shell cat contract/rust-toolchain)

prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rustfmt --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rust-src --toolchain ${PINNED_TOOLCHAIN}

.PHONY:	build-contract
build-contract:
	cd contract && RUSTFLAGS="-C target-cpu=mvp" cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -p cowl-vesting

setup-test: build-contract
	mkdir -p tests/wasm
	cp ./target/wasm32-unknown-unknown/release/cowl_vesting.wasm tests/wasm
	cp ../casper/cep18/target/wasm32-unknown-unknown/release/cowl_cep18.wasm tests/wasm

test: setup-test
	cd tests && cargo test

clippy:
	cd contract && cargo clippy --bins --target wasm32-unknown-unknown -Z build-std=std,panic_abort -- -D warnings
	cd contract && cargo clippy --lib --target wasm32-unknown-unknown -Z build-std=std,panic_abort -- -D warnings
	cd contract && cargo clippy --lib --target wasm32-unknown-unknown -Z build-std=std,panic_abort --no-default-features -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd contract && cargo fmt -- --check
	cd tests && cargo +$(PINNED_TOOLCHAIN) fmt -- --check

format:
	cd contract && cargo fmt
	cd tests && cargo +$(PINNED_TOOLCHAIN) fmt

clean:
	cd contract && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm
