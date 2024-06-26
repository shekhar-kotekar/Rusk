# Rusk
NiFi equivalent built using Rust. Architecture can be found [here](https://docs.google.com/presentation/d/1vFsGreuPf5521KDLQnLpkzTRDoSvygRhhJjB9mcVgaA/edit#slide=id.p)

## Getting started
- Make sure that Make, git and Rust is installed.
- Execute `make run` command which will compile, format, check, execute unit and integration tests and then run main module
- Execute `make test` to test entire project
- Execute `make release` to create a release mode version of the project 

## Rusk Web module
Accepts requests from UI and takes actions like adding a processor, connecting 2 processors, etc.
Execute `make build_web` command build Docker image

## Useful commands:
- To add a new library package, Execute `cargo new --lib <PACKAGE_NAME> --vcs none`
- To run an individual package within a workspace, execute `cargo run -p <MODULE NAME>`. Example : `cargo run -p rusk_web`

## Plan
- [x] Create simplest possible processor
- [ ] Write code with unit tests to verify if a processor can send message to other
- [ ] Create simplest possible data structure which will hold processors and connection between them
- [ ] Understand how content repository works (Flowfile does not store payload/content within itself, it keeps a pointer to content repository)
- [ ] Implement content repository
- [ ] Use content repository to implement flow file
- [ ] Use Flow file to implement simplest "echo" processor (without attributes)

### scratch pad
rough idea about how to create and manage processors and connections between them
- channels / pipe / queue is the key
- When connection is made between two processors
    - create a new MPSC channel
    - add tx to the vector of tx in source processor
    - add rx to the vector of rx in receving processor
- When connection is deleted between two processors
    - remove tx from the vector of tx in source processor
    - remove rx from the vector of rx in receving processor
    - if no other tx and rx using the channel then flush, close, delete the channel
- questions
    - who will monitorg list of channels and connctions?

## References:
- NiFi docs : https://nifi.apache.org/docs/nifi-docs/html/nifi-in-depth.html#intro
