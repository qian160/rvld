
TESTS := $(wildcard tests/*.sh)

build:
	@cargo build
	@ln -sf ./target/debug/rvld ld

test: build
	@$(MAKE) $(TESTS)
	@printf '\e[32mPassed all tests\e[0m\n'

$(TESTS):
	@echo 'Testing' $@
	@./$@
	@printf '\e[32mOK\e[0m\n'

clean:
	cargo clean
	rm -rf ld out/

.PHONY: build clean test $(TESTS)
