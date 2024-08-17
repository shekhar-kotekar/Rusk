# Rusk
NiFi equivalent built using Rust. Architecture diagram and other high level details are [here](https://docs.google.com/presentation/d/1vFsGreuPf5521KDLQnLpkzTRDoSvygRhhJjB9mcVgaA/edit#slide=id.g2e768e227f1_0_6)

## Getting started
- Make sure that Make, git and Rust is installed.
- Execute `make run` command which will compile, format, check, execute unit and integration tests and then run main module
- Execute `make test` to test entire project
- Execute `make release` to create a release mode version of the project 

### Local development
#### Local testing
- Execute `make build PACKAGE=<package name without rust_prefix>` command. For example, to build `rusk_main` package, execute `make build PACKAGE=main`. Similarly we can execute `make deploy` command to build and deploy image in local `kind` based cluster.
- To Execute unit test for a particular package within a workspace, execute `make test PACKAGE=<package name>`. Example : `make test PACKAGE=main`
- To execute unit tests for a single module use this command: `cargo test in_memory_processor -- --nocapture` where `in_memory_processor` is a name of the module which we want to test.

## Rusk Main module
Accepts requests from UI and takes actions like adding a processor, connecting 2 processors, etc.
- Execute `make deploy PACKAGE=main` command build Docker image and deploy it in local `kind`cluster.
- `curl -v http://localhost:30002/is_alive` for aliveness probe
- `curl -v http://localhost:30002/cluster/get_info` to get cluster information
- 30002 port becomes available only when we have created `kind cluster`using `k8s/kind-local-registry.sh` script.

## Useful commands:
- To add a new library package, execute `cargo new --lib <PACKAGE_NAME> --vcs none`

## Content Repository
For producer processors, instead of processor creating FlowFile by itself, it will send the content or location of content (in case of file) to the content repositry. Content repository will create a flow file and send it to the processor after which processor starts using it.

- Content repository will run as another process, possibly in a separate pod so that we can scale-out if necessary and to keep all the modules decoupled as much as possible.
- We can run content repository in core Rusk itself and use MPSC channels for communication between processor and content repository but it will make modules tightly coupled and in case of a crash all the modules will crash.

## References:
- NiFi docs : https://nifi.apache.org/docs/nifi-docs/html/nifi-in-depth.html#intro
