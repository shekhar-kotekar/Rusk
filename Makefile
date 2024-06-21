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
