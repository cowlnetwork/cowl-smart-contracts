SUBDIRS := casper/cep18 cowl-vesting cowl-swap cowl-cli

.PHONY: build clean prepare setup-test test clippy check-lint

# Prepare step for all subprojects
prepare:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) prepare;)

# Clean all subprojects
clean:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) clean;)

# Build all subprojects with conditional targets for cowl-cli
build:
	@$(foreach dir,$(SUBDIRS), \
		$(if $(filter cowl-cli,$(dir)), \
			$(MAKE) -C $(dir) build, \
			$(if $(filter casper/cep18,$(dir)), \
				$(MAKE) -C $(dir) build-all-contracts, \
				$(MAKE) -C $(dir) build-contract) ;) )

# Test all subprojects with dowloaded wasm files
setup-test:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) setup-test;)

# Test all subprojects with lastest compiled wasm files
setup-test-dev: build copy-wasm

copy-wasm:
	@$(foreach dir, $(SUBDIRS), \
		$(if $(filter-out cowl-cli, $(dir)), \
			$(MAKE) -C $(dir) copy-wasm;))

	mkdir -p ./cowl-cli/wasm

	cp ./casper/cep18/target/wasm32-unknown-unknown/release/cowl_cep18.wasm ./cowl-vesting/tests/wasm
	cp ./casper/cep18/target/wasm32-unknown-unknown/release/cowl_cep18.wasm ./cowl-swap/tests/wasm
	cp ./casper/cep18/target/wasm32-unknown-unknown/release/cowl_cep18.wasm ./cowl-cli/wasm

	cp ./cowl-vesting/target/wasm32-unknown-unknown/release/*.wasm ./cowl-swap/tests/wasm
	cp ./cowl-vesting/target/wasm32-unknown-unknown/release/*.wasm ./cowl-cli/wasm

	cp ./cowl-swap/target/wasm32-unknown-unknown/release/*.wasm ./cowl-cli/wasm

test:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) test;)

test-dev: setup-test-dev
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) test-dev;)

check-lint:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) check-lint;)