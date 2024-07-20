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
	docker build -t rust_web:latest -f rusk_web/Dockerfile .
	# --progress plain
	@echo "web server built successfully!"
	@echo "run 'docker run -it -p 8080:5056 rust_web:latest' to start the server"

run_content_repo: test
	@echo $$CONFIG_FILE_PATH
	cargo run --package content_repository

test_content_repo: prepare
	cargo test -p content_repository --bin content_repository

build_content_repo: prepare
	docker build -t content_repo:latest -f content_repository/Dockerfile .
	@echo "content repository built successfully!"
	@echo "run 'docker run -it -p 8081:5057 content_repo:latest' to start the server"
	