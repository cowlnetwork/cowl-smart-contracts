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
			$(MAKE) -C $(dir) build-contract) ;)

# Test all subprojects
setup-test: build
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) setup-test;)

test:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) test;)

check-lint:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) check-lint;)