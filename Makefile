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

deploy_common:
	@echo "INFO: Make sure you have enabled minikube registry AND started alpine container as mentioned in README.md"
	kubectl apply -f k8s/common.yaml

deploy_client: deploy_common
	@echo "INFO: Make sure you have enabled minikube registry AND started alpine container as mentioned in README.md"
	@echo "WARNING: Make sure you have built the client image using 'make build_client' from rusk_client project"
	
	docker push localhost:5000/rusk_client:latest
	@echo "Client pushed to minikube local registry"

	kubectl apply -f k8s/rusk_client.yaml
	@echo "Client is running on http://localhost:8080"

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
	docker build --tag localhost:5000/rusk_content_repo:latest -f content_repository/Dockerfile .
	docker push localhost:5000/rusk_content_repo:latest
	
	@echo "content repository built successfully!"
	@echo "run 'docker run -it -p 8081:5057 content_repo:latest' to start the server"

deploy_content_repo: build_content_repo deploy_common
	kubectl apply -f k8s/content_repository.yaml 
	@echo "content repository deployed successfully!"

build_main: prepare
	docker build -t localhost:5000/rusk_main:latest -f main/Dockerfile .
	docker push localhost:5000/rusk_main:latest
	
	@echo "Rusk main built successfully!"

deploy_main: build_main deploy_common
	kubectl apply -f k8s/main.yaml
	@echo "Rusk main deployed successfully!"

build_all: build_web build_content_repo build_main
	@echo "All services built successfully!"

deploy_all: build_all
	@echo "INFO: all modules deployed successfully!"

delete_all:
	kubectl delete -f k8s/content_repository.yaml
	kubectl delete -f k8s/main.yaml
	kubectl delete -f k8s/rusk_client.yaml
	kubectl delete -f k8s/common.yaml
	@echo "All deployments deleted successfully!"

start_local_container_registry:
	kubectl config use-context minikube
	docker run --rm -it --network=host alpine ash -c "apk add socat && socat TCP-LISTEN:5000,reuseaddr,fork TCP:$(minikube ip):5000"
	@echo "Local container registry started successfully!"
	