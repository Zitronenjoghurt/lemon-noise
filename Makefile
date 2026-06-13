.PHONY: up down build dev check app web server fmt

up:
	docker compose -f docker/docker-compose.yml up -d

down:
	docker compose -f docker/docker-compose.yml down

build:
	docker image prune -f
	docker compose -f docker/docker-compose.yml build

dev:
	docker compose -f docker/docker-compose.yml --profile dev up server-dev

fmt:
	cargo fmt --all

check:
	cargo fmt --all --check
	cargo clippy -p lemon-noise --all-targets -- -D warnings
	cargo clippy -p lemon-noise --all-targets --features audio,persistence -- -D warnings
	cargo clippy -p lemon-noise-server --all-targets -- -D warnings
	cargo clippy -p lemon-noise-app --all-targets -- -D warnings
	cargo clippy -p lemon-noise-app --target wasm32-unknown-unknown -- -D warnings
	cargo test -p lemon-noise --features audio,persistence
	cargo test -p lemon-noise-app
	cargo build -p lemon-noise-app --target wasm32-unknown-unknown

app:
	cargo run -p lemon-noise-app

web:
	trunk serve --open
