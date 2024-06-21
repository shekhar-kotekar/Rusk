FROM docker.io/library/rust:alpine3.20 as builder

WORKDIR /rusk_web_app
COPY . /rusk_web_app/

RUN rustup default stable
RUN apk add --no-cache musl-dev libpq gcc

RUN cargo test && cargo build --release

FROM scratch
COPY --from=builder /rusk_web_app/target/release/rusk_web /rusk_web
CMD [ "rusk_web" ]
