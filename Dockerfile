FROM rust:1.80-slim AS builder
WORKDIR /usr/src/engram
COPY . .
RUN apt-get update && apt-get install -y pkg-config libssl-dev gcc
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/engram/target/release/engram-cli /app/engram-cli
ENTRYPOINT ["/app/engram-cli"]
