FROM docker.io/library/rust:alpine3.20 AS builder

RUN rustup default stable
RUN apk add --no-cache musl-dev libpq gcc

ARG module_name=rusk_main

WORKDIR /${module_name}
COPY Cargo.toml Cargo.lock /${module_name}/
COPY . /${module_name}/
RUN echo $(ls -ltrh /${module_name})

RUN cargo test --package ${module_name} && \
    cargo build --package ${module_name} --release

RUN echo $(ls -ltrh /${module_name}/target/release/)

FROM scratch
ARG module_name=rusk_main
COPY --from=builder /${module_name}/target/release/${module_name} /rusk_main
COPY ./config.toml /config.toml
CMD [ "/rusk_main" ]
