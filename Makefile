export CONFIG_FILE_PATH := config.toml

IMAGE_REGISTRY := localhost:5001
k8s_context := kind-kind

.PHONY: prepare test run release build_web

prepare:
	cargo fmt && cargo clippy && cargo check
	kubectl config use-context ${k8s_context}

test: prepare
	RUST_LOG=debug cargo test

run: test
	RUST_LOG=warn cargo run

release: test
	cargo build --release

deploy_common:
	@echo
	@echo "INFO: Make sure you have enabled minikube registry AND started alpine container as mentioned in README.md"
	kubectl config use-context ${k8s_context}
	kubectl apply -f k8s/common.yaml
	@echo

deploy_client: deploy_common
	@echo "WARNING: Make sure you have built the client image using 'make build_client' from rusk_client project"
	@echo
	docker push ${IMAGE_REGISTRY}/rusk_client:latest
	@echo
	@echo "INFO: Client pushed to ${IMAGE_REGISTRY} registry"
	@echo
	kubectl apply -f k8s/rusk_client.yaml
	@echo "Client deployed successfully! Access the client using http://localhost:8080 URL in browser."

build_web: prepare
	docker build --tag ${IMAGE_REGISTRY}/rusk_web:latest -f rusk_web/Dockerfile .
	docker push ${IMAGE_REGISTRY}/rusk_web:latest
	@echo "Rusk web server built successfully!"

deploy_web: build_web deploy_common
	kubectl apply -f k8s/rusk_web.yaml
	@echo "web server deployed successfully!"
	@echo "Run curl -v http://localhost:30000/is_alive to check if the server is running"

run_content_repo: test
	@echo $$CONFIG_FILE_PATH
	cargo run --package content_repository

test_content_repo: prepare
	cargo test -p content_repository --bin content_repository

build_content_repo: prepare
	docker build --tag ${IMAGE_REGISTRY}/rusk_content_repo:latest -f content_repository/Dockerfile .
	docker push ${IMAGE_REGISTRY}/rusk_content_repo:latest
	
	@echo "content repository built successfully!"
	@echo "run 'docker run -it -p 8081:5057 content_repo:latest' to start the server"

deploy_content_repo: build_content_repo deploy_common
	kubectl apply -f k8s/content_repository.yaml 
	@echo "content repository deployed successfully!"

build_main: prepare
	docker build -t ${IMAGE_REGISTRY}/rusk_main:latest -f main/Dockerfile .
	docker push ${IMAGE_REGISTRY}/rusk_main:latest
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
