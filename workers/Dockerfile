FROM rust:1.79 as builder
WORKDIR /app
COPY Cargo.toml ./
COPY api/Cargo.toml api/Cargo.toml
COPY workers/Cargo.toml workers/Cargo.toml
RUN mkdir -p api/src workers/src && \
    echo 'fn main(){}' > api/src/main.rs && \
    echo 'fn main(){}' > workers/src/main.rs && \
    cargo build -p trustsystem-workers --release || true
COPY . .
RUN cargo build -p trustsystem-workers --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /srv
COPY --from=builder /app/target/release/trustsystem-workers /usr/local/bin/workers
ENV RUST_LOG=info
CMD ["/usr/local/bin/workers"]


