FROM rust:1.85-slim AS builder

WORKDIR /app
COPY . .

# Build without GPU (pure CPU path — for MCP tool inspection)
ENV ENGRAM_NO_GPU=1
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release -p engram-server

# ── Runtime stage ──────────────────────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/engram /usr/local/bin/engram

# Default store inside the container
RUN mkdir -p /data/engram

ENV ENGRAM_STORE=/data/engram
EXPOSE 3000

# Run MCP server on stdio (standard MCP transport)
ENTRYPOINT ["engram", "mcp", "--store", "/data/engram"]
