FROM docker.io/library/rust:alpine3.20 as builder

RUN rustup default stable
RUN apk add --no-cache musl-dev libpq gcc

WORKDIR /rusk_web_app
COPY Cargo.toml Cargo.lock /rusk_web_app/
COPY . /rusk_web_app/
RUN echo $(ls -ltrh /rusk_web_app)

# RUN cargo test && cargo build --target x86_64-unknown-linux-musl --release
RUN cargo clean && \
    cargo test --package commons && \
    cargo test --package rusk_web && \
    cargo build --release

RUN echo $(ls -ltrh /rusk_web_app/target/release/)

FROM scratch
COPY --from=builder /rusk_web_app/target/release/rusk_web /rusk_web
CMD [ "/rusk_web" ]
