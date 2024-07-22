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
	@echo "INFO: Make sure you have enabled minikube registry AND started alpine container as mentioned in README.md"
	docker build --tag localhost:5000/rusk_content_repo:latest -f content_repository/Dockerfile .
	docker push localhost:5000/rusk_content_repo:latest
	@echo "content repository built successfully!"
	@echo "run 'docker run -it -p 8081:5057 content_repo:latest' to start the server"

deploy_content_repo: build_content_repo
	kubectl apply -f k8s/common.yaml
	kubectl apply -f k8s/content_repository.yaml 
	@echo "content repository deployed successfully!"

build_main: prepare
	docker build -t rusk_main:latest -f main/Dockerfile .
	@echo "Rusk main built successfully!"

build_all: build_web build_content_repo build_main
	@echo "All services built successfully!"
	