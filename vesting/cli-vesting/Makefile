prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy
	rustup component add rustfmt

.PHONY:	build
build:
	cargo build
	cargo build --lib --target wasm32-unknown-unknown

test:
	cd tests && cargo test -- --test-threads=1 --nocapture

clippy:
	cargo clippy --bins
	cargo clippy --lib --target wasm32-unknown-unknown
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cargo fmt -- --check
	cd tests && cargo fmt -- --check

clean:
	cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm