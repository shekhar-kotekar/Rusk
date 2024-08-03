# Rusk
NiFi equivalent built using Rust. Architecture diagram and other high level details are [here](https://docs.google.com/presentation/d/1vFsGreuPf5521KDLQnLpkzTRDoSvygRhhJjB9mcVgaA/edit#slide=id.g2e768e227f1_0_6)

## Getting started
- Make sure that Make, git and Rust is installed.
- Execute `make run` command which will compile, format, check, execute unit and integration tests and then run main module
- Execute `make test` to test entire project
- Execute `make release` to create a release mode version of the project 

### Local development
We use Minikube to check modules in local so make sure that Minikube is installed and "registry" addon is enabled.
Execute below commands to enable and start local image registry:
```
# make sure that current context is set to kind-kind
kubectl config current-context
```

## Rusk Web module
Accepts requests from UI and takes actions like adding a processor, connecting 2 processors, etc.
Execute `make build_web` command build Docker image

## Useful commands:
- To add a new library package, execute `cargo new --lib <PACKAGE_NAME> --vcs none`
- To run an individual package within a workspace, execute `cargo run -p <MODULE NAME>`. Example : `cargo run -p rusk_web`
- To Execute unit test for a particular module, execute `cargo test --package <module name>`. Example : `cargo test --package rusk_web`

## Content Repository
For producer processors, instead of processor creating FlowFile by itself, it will send the content or location of content (in case of file) to the content repositry. Content repository will create a flow file and send it to the processor after which processor starts using it.

- Content repository will run as another process, possibly in a separate pod so that we can scale-out if necessary and to keep all the modules decoupled as much as possible.
- We can run content repository in core Rusk itself and use MPSC channels for communication between processor and content repository but it will make modules tightly coupled and in case of a crash all the modules will crash.

## References:
- NiFi docs : https://nifi.apache.org/docs/nifi-docs/html/nifi-in-depth.html#intro
- Minikube local registry - https://minikube.sigs.k8s.io/docs/handbook/registry/
- Minikube docker on MacOs - https://minikube.sigs.k8s.io/docs/handbook/registry/#docker-on-macos
