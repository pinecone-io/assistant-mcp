FROM rust:1.85-slim AS builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY rust-sdk ./rust-sdk/
COPY Cargo.toml Cargo.lock ./

# Create a dummy project for dependency caching https://stackoverflow.com/a/58474618/2318775
RUN mkdir -p src && \
    echo "fn main() {println!(\"Dummy build\");}" > src/main.rs && \
    cargo build --release --lib || true && \
    rm -rf src

# Now copy the actual source code and build the application
COPY src ./src/
RUN cargo build --release

FROM debian:bookworm-slim AS release

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/assistant-mcp /app/assistant-mcp

ENV RUST_LOG=info

ENTRYPOINT ["/app/assistant-mcp"] 
