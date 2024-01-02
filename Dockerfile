FROM rust:slim-bookworm
LABEL org.opencontainers.image.source https://github.com/molguin92/agh-exporter-rs

RUN apt-get update && apt-get install -y libc-dev pkg-config libssl-dev openssl

COPY . /agh-exporter-rs
WORKDIR /agh-exporter-rs

RUN cargo build --release
ENTRYPOINT ["target/release/agh-exporter-rs"]
