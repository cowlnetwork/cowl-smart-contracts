SUBDIRS := casper/cep18 vesting swap

.PHONY: build clean prepare $(SUBDIRS)

# Prepare step for all subprojects
prepare:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) prepare;)
	$(MAKE) -C vesting/cli-vesting prepare

# Clean all subprojects
clean:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) clean;)
	$(MAKE) -C vesting/cli-vesting clean

# Build all subprojects
build:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) build-contract;)
	$(MAKE) -C vesting/cli-vesting build

# Test all subprojects
setup-test: build 
	$(MAKE) -C vesting setup-test

test:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) test;)
	$(MAKE) -C vesting/cli-vesting test

clippy:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) clippy;)
	$(MAKE) -C vesting/cli-vesting clippy

check-lint:
	@$(foreach dir,$(SUBDIRS),$(MAKE) -C $(dir) check-lint;)
	$(MAKE) -C vesting/cli-vesting check-lint
