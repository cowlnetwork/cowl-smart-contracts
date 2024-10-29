CARGO = cargo
CARGO_FLAGS = --release
WASM_TARGET = wasm32-unknown-unknown
WASM_BUILD_PATH = target/$(WASM_TARGET)/release
CONTRACT_NAME = cowl_token

.PHONY: all prepare build test clean

all: build test

prepare:
	rustup target add $(WASM_TARGET)
	$(CARGO) install cargo-casper
	$(CARGO) install wasm-strip

build:
	$(CARGO) build $(CARGO_FLAGS) --target $(WASM_TARGET)
	wasm-strip $(WASM_BUILD_PATH)/$(CONTRACT_NAME).wasm

test: build
	$(CARGO) test

test-with-logging: build
	$(CARGO) test -- --nocapture

check:
	$(CARGO) check
	$(CARGO) clippy --all-targets -- -D warnings

format:
	$(CARGO) fmt --all -- --check

clean:
	$(CARGO) clean
	rm -rf target
