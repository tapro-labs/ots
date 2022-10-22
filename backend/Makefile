default: dev

# key for local development only
dev:
	ROCKET_SECRET_KEY="qpTdCblYgdwBv8TRyKpG0vgQ5rOSLq3aKDZWqQgU6Kk=" ROCKET_PROFILE=debug cargo watch -x run

# key for local development only
docker_dev:
	ROCKET_SECRET_KEY="qpTdCblYgdwBv8TRyKpG0vgQ5rOSLq3aKDZWqQgU6Kk=" ROCKET_PROFILE=docker cargo watch -x run

lint:
	cargo clippy -- -D warnings -A clippy::let_unit_value

format:
	cargo clippy --allow-dirty --allow-staged --fix
