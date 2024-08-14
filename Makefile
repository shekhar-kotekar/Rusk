export CONFIG_FILE_PATH := config.toml

IMAGE_REGISTRY := localhost:5001
k8s_context := kind-kind

.PHONY: prepare test run release build_web

set_kind_context:
	kubectl config use-context ${k8s_context}

prepare:
	@if [ -z "$(PACKAGE)" ]; then \
        echo "Error: PACKAGE variable is not set"; \
        exit 1; \
    fi
	@echo "Preparing $(PACKAGE) package"
	cargo fmt && cargo clippy && cargo check

test: prepare
	@echo
	@echo "Running tests for $(PACKAGE) package"
	RUST_LOG=debug cargo test --package rusk_$(PACKAGE) -- --nocapture

run: test
	RUST_LOG=warn cargo run

release: test
	cargo build --release

deploy_common: set_kind_context
	kubectl apply -f k8s/common.yaml

deploy_client: deploy_common
	@echo "WARNING: Make sure you have built the client image using 'make build_client' from rusk_client project"
	@echo
	docker push ${IMAGE_REGISTRY}/rusk_client:latest
	@echo
	@echo "INFO: Client pushed to ${IMAGE_REGISTRY} registry."
	@echo
	kubectl apply -f k8s/rusk_client.yaml
	@echo "Client deployed successfully! Access the client using http://localhost:8080 URL in browser."

build: prepare set_kind_context
	@echo "INFO: Building $(PACKAGE) package"
	docker build --tag ${IMAGE_REGISTRY}/rusk_$(PACKAGE):latest -f rusk_$(PACKAGE)/Dockerfile .
	docker push ${IMAGE_REGISTRY}/rusk_$(PACKAGE):latest
	@echo "$(PACKAGE) built successfully!"

deploy: build deploy_common
	@echo "INFO: Deploying $(PACKAGE) package"
	kubectl apply -f k8s/rusk_$(PACKAGE).yaml
	@echo "$(PACKAGE) deployed successfully!"

delete_all: set_kind_context
	kubectl delete -f k8s/rusk_content_repo.yaml
	kubectl delete -f k8s/rusk_main.yaml
	kubectl delete -f k8s/rusk_client.yaml
	kubectl delete -f k8s/common.yaml
	@echo "All deployments deleted successfully!"
