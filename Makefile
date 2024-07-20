export CONFIG_FILE_PATH := config.toml

.PHONY: prepare test run release build_web

prepare:
	cargo fmt && cargo clippy && cargo check

test: prepare
	RUST_LOG=debug cargo test

run: test
	RUST_LOG=warn cargo run

release: test
	cargo build --release

build_web: prepare
	cargo test --package commons && \
    cargo test --package rusk_web
	# --progress plain
	docker build -t rust_web:latest -f rusk_web/Dockerfile .
	@echo "web server built successfully!"
	@echo "run 'docker run -it -p 8080:5056 rust_web:latest' to start the server"

run_content_repo: test
	@echo $$CONFIG_FILE_PATH
	cargo run --package content_repository

test_content_repo: prepare
	cargo test -p content_repository --bin content_repository
	