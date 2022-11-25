# Optionally include .env file
-include .env

APP_FEATURES_ENABLED = $(shell echo ${ENABLED_FEATURES} | sed -E 's/([^,]+)/\1_feature/g')

ifneq ($(APP_FEATURES_ENABLED),)
	RELEASE_FLAG = --features $(APP_FEATURES_ENABLED)
endif

default: build

build:
	cargo build --release $(RELEASE_FLAG)

# key for local development only
dev:
	cargo watch $(RELEASE_FLAG) -x run

lint:
	cargo clippy -- -D warnings -A clippy::let_unit_value

format:
	cargo clippy --allow-dirty --allow-staged --fix
