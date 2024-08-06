export CONFIG_FILE_PATH := config.toml

IMAGE_REGISTRY := localhost:5001
k8s_context := kind-kind

.PHONY: prepare test run release build_web

prepare:
	cargo fmt && cargo clippy && cargo check
	kubectl config use-context ${k8s_context}

test: prepare
	@echo
	@echo "Running tests for $(PACKAGE) package"
	RUST_LOG=debug cargo test --package rusk_$(PACKAGE) -- --nocapture

run: test
	RUST_LOG=warn cargo run

release: test
	cargo build --release

deploy_common:
	kubectl config use-context ${k8s_context}
	kubectl apply -f k8s/common.yaml

deploy_client: deploy_common
	@echo "WARNING: Make sure you have built the client image using 'make build_client' from rusk_client project"
	@echo
	docker push ${IMAGE_REGISTRY}/rusk_client:latest
	@echo
	@echo "INFO: Client pushed to ${IMAGE_REGISTRY} registry"
	@echo
	kubectl apply -f k8s/rusk_client.yaml
	@echo "Client deployed successfully! Access the client using http://localhost:8080 URL in browser."

build: prepare
	@echo "INFO: Building $(PACKAGE) package"
	docker build --tag ${IMAGE_REGISTRY}/rusk_$(PACKAGE):latest -f rusk_$(PACKAGE)/Dockerfile .
	docker push ${IMAGE_REGISTRY}/rusk_$(PACKAGE):latest
	@echo "$(PACKAGE) built successfully!"

deploy: build deploy_common
	@echo "INFO: Deploying $(PACKAGE) package"
	kubectl apply -f k8s/rusk_$(PACKAGE).yaml
	@echo "$(PACKAGE) deployed successfully!"

run_content_repo: test
	@echo $$CONFIG_FILE_PATH
	cargo run --package content_repository

test_content_repo: prepare
	cargo test -p content_repository --bin content_repository

build_all:
	@echo "NOT IMPLEMENTED!"

deploy_all: build_all
	@echo "INFO: all modules deployed successfully!"

delete_all:
	kubectl delete -f k8s/rusk_content_repo.yaml
	kubectl delete -f k8s/rusk_main.yaml
	kubectl delete -f k8s/rusk_client.yaml
	kubectl delete -f k8s/common.yaml
	@echo "All deployments deleted successfully!"
