STACK_NAME ?= sam-rust-inventory
FUNCTIONS := process-s3-event process-sqs-event

ARCH := aarch64-unknown-linux-gnu
ARCH_SPLIT = $(subst -, ,$(ARCH))

.PHONY: build deploy tests

all: build tests-unit deploy tests-integ
ci: build tests-unit

setup:
ifeq (,$(shell which rustc))
	$(error "Could not find Rust compiler, please install it")
endif
ifeq (,$(shell which cargo))
	$(error "Could not find Cargo, please install it")
endif
ifeq (,$(shell which zig))
	$(error "Could not find Zig compiler, please install it")
endif
	cargo install cargo-lambda
ifeq (,$(shell which sam))
	$(error "Could not find SAM CLI, please install it")
endif


build:
	cargo lambda build --release --target $(ARCH)

deploy:
	if [ -f samconfig.toml ]; \
		then sam deploy --stack-name $(STACK_NAME); \
		else sam deploy -g --stack-name $(STACK_NAME); \
	fi
destroy:
	sam delete --stack-name $(STACK_NAME)
tests-unit:
	cargo test --lib --bins

tests-integ:
	RUST_BACKTRACE=1 API_URL=$$(aws cloudformation describe-stacks --stack-name $(STACK_NAME) \
		--query 'Stacks[0].Outputs[?OutputKey==`ApiUrl`].OutputValue' \
		--output text) cargo test

tests-load:
	API_URL=$$(aws cloudformation describe-stacks --stack-name $(STACK_NAME) \
		--query 'Stacks[0].Outputs[?OutputKey==`ApiUrl`].OutputValue' \
		--output text) artillery run tests/load-test.yml
