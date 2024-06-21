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
	# --progress plain
	docker build -t rust_web:latest -f rusk_web/Dockerfile .
	@echo "web server built successfully!"
	@echo "run 'docker run -p 8080:5056 rust_web:latest' to start the server"