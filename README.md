# Rusk
NiFi equivalent built using Rust. Architecture can be found [here](https://docs.google.com/presentation/d/1vFsGreuPf5521KDLQnLpkzTRDoSvygRhhJjB9mcVgaA/edit#slide=id.p)

## Getting started
- Make sure that Make, git and Rust is installed.
- Execute `make run` command which will compile, format, check, execute unit and integration tests and then run main module
- Execute `make test` to test entire project
- Execute `make release` to create a release mode version of the project 

### Local development
We use Minikube to check modules in local so make sure that Minikube is installed and "registry" addon is enabled.
Execute below commands to enable and start local image registry:
```
minikube addons enable registry

# this command keeps current terminal engaged so open another terminal
kubectl port-forward --namespace kube-system service/registry 5000:80

# this command runs docker container in interactive mode so open another terminal
docker run --rm -it --network=host alpine ash -c "apk add socat && socat TCP-LISTEN:5000,reuseaddr,fork TCP:host.docker.internal:5000"

curl http://localhost:5000/v2/_catalog
```

Once local image registry is enabled and started tag docker image and push using below commands:
```
docker tag rusk_content_repo localhost:5000/rusk_content_repo:latest
docker push localhost:5000/rusk_content_repo:latest
```

## Rusk Web module
Accepts requests from UI and takes actions like adding a processor, connecting 2 processors, etc.
Execute `make build_web` command build Docker image

## Useful commands:
- To add a new library package, Execute `cargo new --lib <PACKAGE_NAME> --vcs none`
- To run an individual package within a workspace, execute `cargo run -p <MODULE NAME>`. Example : `cargo run -p rusk_web`

## Plan
- [x] Create simplest possible processor
- [x] Write code with unit tests to verify if a processor can send message to other
- [x] Create simplest possible data structure which will hold processors and connection between them
- [x] Understand how content repository works (Flowfile does not store payload/content within itself, it keeps a pointer to content repository)
- [ ] Implement content repository
- [ ] Use content repository to implement flow file
- [ ] Use Flow file to implement simplest "echo" processor (without attributes)

## Content Repository
For producer processors, instead of processor creating FlowFile by itself, it will send the content or location of content (in case of file) to the content repositry. Content repository will create a flow file and send it to the processor after which processor starts using it.

- Content repository will run as another process, possibly in a separate pod so that we can scale-out if necessary and to keep all the modules decoupled as much as possible.
- We can run content repository in core Rusk itself and use MPSC channels for communication between processor and content repository but it will make modules tightly coupled and in case of a crash all the modules will crash.

## References:
- NiFi docs : https://nifi.apache.org/docs/nifi-docs/html/nifi-in-depth.html#intro
