# Optionally include .env file
-include .env

BUILD_COMMAND = cargo
TARGET=x86_64-unknown-linux-gnu
APP_FEATURES_ENABLED = $(shell echo ${ENABLED_FEATURES} | sed -E 's/([^,]+)/\1_feature/g')

ifeq ($(ARM),true)
  BUILD_COMMAND = cross
  TARGET=aarch64-unknown-linux-gnu
endif

ifneq ($(APP_FEATURES_ENABLED),)
	RELEASE_FLAG = --features $(APP_FEATURES_ENABLED)
endif

default: build

build:
	$(BUILD_COMMAND) build --target $(TARGET) --release $(RELEASE_FLAG)

# key for local development only
dev:
	cargo watch $(RELEASE_FLAG) -x run

lint:
	cargo clippy -- -D warnings -A clippy::let_unit_value

format:
	cargo clippy --allow-dirty --allow-staged --fix
